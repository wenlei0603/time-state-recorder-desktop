use std::{
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};

use tauri::{AppHandle, Manager, RunEvent, State};
use tauri_plugin_shell::{ShellExt, process::CommandChild};

mod config;
mod secrets;

use config::{
    ConfigPaths, DesktopConfig, DesktopConfigStore, DesktopConfigView, ProviderTestInput,
    ProviderTestResult, SecretStatus,
};
use secrets::{DpapiSecretStore, SecretKey, SecretStore};

const COLLECTOR_SIDECAR: &str = "tsr-collector";
const DEFAULT_COLLECTOR_ADDR: &str = "127.0.0.1:4317";

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DesktopHealth {
    status: &'static str,
    product_name: &'static str,
    version: &'static str,
    collector: CollectorDesktopStatus,
}

#[tauri::command]
fn desktop_health(state: State<'_, CollectorState>) -> DesktopHealth {
    DesktopHealth {
        status: "ok",
        product_name: "Time State Recorder Desktop",
        version: env!("CARGO_PKG_VERSION"),
        collector: state.snapshot(),
    }
}

#[tauri::command]
fn collector_status(state: State<'_, CollectorState>) -> CollectorDesktopStatus {
    state.snapshot()
}

#[tauri::command]
fn start_collector(
    app: AppHandle,
    state: State<'_, CollectorState>,
) -> Result<CollectorDesktopStatus, String> {
    start_collector_from_app(&app, &state)
}

#[tauri::command]
fn stop_collector(state: State<'_, CollectorState>) -> Result<CollectorDesktopStatus, String> {
    state.stop()
}

#[tauri::command]
fn get_desktop_config(app: AppHandle) -> Result<DesktopConfigView, String> {
    load_config_view(&app)
}

#[tauri::command]
fn save_desktop_config(app: AppHandle, config: DesktopConfig) -> Result<DesktopConfigView, String> {
    let paths = config_paths_from_app(&app)?;
    DesktopConfigStore::new(paths).save(&config)?;
    load_config_view(&app)
}

#[tauri::command]
fn set_ai_provider_api_key(app: AppHandle, secret: String) -> Result<SecretStatus, String> {
    let paths = config_paths_from_app(&app)?;
    let store = DpapiSecretStore::new(paths.secret_dir());
    store.put(SecretKey::AiProviderApiKey, &secret)?;
    store.status(SecretKey::AiProviderApiKey)
}

#[tauri::command]
fn clear_ai_provider_api_key(app: AppHandle) -> Result<SecretStatus, String> {
    let paths = config_paths_from_app(&app)?;
    let store = DpapiSecretStore::new(paths.secret_dir());
    store.delete(SecretKey::AiProviderApiKey)?;
    store.status(SecretKey::AiProviderApiKey)
}

#[tauri::command]
fn test_ai_provider_connection(app: AppHandle) -> Result<ProviderTestResult, String> {
    let paths = config_paths_from_app(&app)?;
    let config = DesktopConfigStore::new(paths.clone()).load_or_default()?;
    let secret_status =
        DpapiSecretStore::new(paths.secret_dir()).status(SecretKey::AiProviderApiKey)?;
    ProviderTestInput {
        config,
        secret_status,
    }
    .build_result()
}

fn main() {
    let app = tauri::Builder::default()
        .manage(CollectorState::default())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = app.state::<CollectorState>();
            if let Err(error) = start_collector_from_app(&handle, &state) {
                state.set_error(default_collector_status(Some(error)));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            desktop_health,
            collector_status,
            start_collector,
            stop_collector,
            get_desktop_config,
            save_desktop_config,
            set_ai_provider_api_key,
            clear_ai_provider_api_key,
            test_ai_provider_connection
        ])
        .build(tauri::generate_context!())
        .expect("error while running Time State Recorder desktop");

    app.run(|app_handle, event| {
        if matches!(event, RunEvent::ExitRequested { .. } | RunEvent::Exit) {
            let state = app_handle.state::<CollectorState>();
            let _ = state.stop();
        }
    });
}

#[derive(Debug, Clone)]
struct CollectorLaunchConfig {
    data_dir: PathBuf,
    db_path: PathBuf,
    blocker_config: PathBuf,
    addr: SocketAddr,
    poll_ms: u64,
}

impl CollectorLaunchConfig {
    fn new(data_dir: PathBuf, addr: SocketAddr, poll_ms: u64) -> Self {
        Self {
            db_path: data_dir.join("local.sqlite3"),
            blocker_config: data_dir.join("blocker_config.json"),
            data_dir,
            addr,
            poll_ms,
        }
    }

    fn from_app(app: &AppHandle) -> Result<Self, String> {
        let paths = config_paths_from_app(app)?;
        let config = DesktopConfigStore::new(paths).load_or_default()?;
        let addr = format!("127.0.0.1:{}", config.system.api_port)
            .parse()
            .map_err(|error| format!("Invalid collector address: {error}"))?;
        Ok(Self::new(
            config.storage.data_dir.clone(),
            addr,
            config.capture.poll_ms,
        ))
    }

    fn args(&self) -> Vec<String> {
        vec![
            "serve".into(),
            "--db".into(),
            path_arg(&self.db_path),
            "--addr".into(),
            self.addr.to_string(),
            "--poll-ms".into(),
            self.poll_ms.to_string(),
            "--blocker-config".into(),
            path_arg(&self.blocker_config),
        ]
    }

    fn api_url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn data_dir(&self) -> String {
        path_arg(&self.data_dir)
    }
}

#[derive(serde::Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct CollectorDesktopStatus {
    status: String,
    managed: bool,
    pid: Option<u32>,
    api_url: String,
    data_dir: Option<String>,
    last_error: Option<String>,
}

struct CollectorRuntime {
    status: CollectorDesktopStatus,
    child: Option<CommandChild>,
}

impl Default for CollectorRuntime {
    fn default() -> Self {
        Self {
            status: default_collector_status(None),
            child: None,
        }
    }
}

#[derive(Default)]
struct CollectorState {
    inner: Mutex<CollectorRuntime>,
}

impl CollectorState {
    fn snapshot(&self) -> CollectorDesktopStatus {
        self.inner
            .lock()
            .map(|runtime| runtime.status.clone())
            .unwrap_or_else(|_| {
                default_collector_status(Some("Collector state lock is poisoned".into()))
            })
    }

    fn set_error(&self, status: CollectorDesktopStatus) {
        if let Ok(mut runtime) = self.inner.lock() {
            runtime.status = status;
            runtime.child = None;
        }
    }

    fn set_external(&self, config: &CollectorLaunchConfig) -> CollectorDesktopStatus {
        let status = CollectorDesktopStatus {
            status: "external".into(),
            managed: false,
            pid: None,
            api_url: config.api_url(),
            data_dir: Some(config.data_dir()),
            last_error: None,
        };
        if let Ok(mut runtime) = self.inner.lock() {
            runtime.status = status.clone();
            runtime.child = None;
        }
        status
    }

    fn set_managed(
        &self,
        config: &CollectorLaunchConfig,
        pid: Option<u32>,
        child: CommandChild,
    ) -> CollectorDesktopStatus {
        let status = CollectorDesktopStatus {
            status: "running".into(),
            managed: true,
            pid,
            api_url: config.api_url(),
            data_dir: Some(config.data_dir()),
            last_error: None,
        };
        if let Ok(mut runtime) = self.inner.lock() {
            runtime.status = status.clone();
            runtime.child = Some(child);
        }
        status
    }

    fn stop(&self) -> Result<CollectorDesktopStatus, String> {
        let mut runtime = self
            .inner
            .lock()
            .map_err(|_| "Collector state lock is poisoned".to_string())?;

        if let Some(child) = runtime.child.take() {
            child
                .kill()
                .map_err(|error| format!("Unable to stop collector: {error}"))?;
        }

        runtime.status = CollectorDesktopStatus {
            status: "stopped".into(),
            managed: false,
            pid: None,
            api_url: runtime.status.api_url.clone(),
            data_dir: runtime.status.data_dir.clone(),
            last_error: None,
        };
        Ok(runtime.status.clone())
    }
}

fn start_collector_from_app(
    app: &AppHandle,
    state: &CollectorState,
) -> Result<CollectorDesktopStatus, String> {
    let config = CollectorLaunchConfig::from_app(app)?;

    if state.snapshot().managed {
        return Ok(state.snapshot());
    }

    if loopback_is_listening(&config.addr) {
        return Ok(state.set_external(&config));
    }

    std::fs::create_dir_all(&config.data_dir)
        .map_err(|error| format!("Unable to create collector data directory: {error}"))?;

    let sidecar = app
        .shell()
        .sidecar(COLLECTOR_SIDECAR)
        .map_err(|error| format!("Unable to prepare collector sidecar: {error}"))?
        .args(config.args());
    let (_rx, child) = sidecar
        .spawn()
        .map_err(|error| format!("Unable to start collector sidecar: {error}"))?;
    let pid = child.pid();

    Ok(state.set_managed(&config, Some(pid), child))
}

fn default_collector_status(last_error: Option<String>) -> CollectorDesktopStatus {
    CollectorDesktopStatus {
        status: if last_error.is_some() {
            "error"
        } else {
            "not_started"
        }
        .into(),
        managed: false,
        pid: None,
        api_url: format!("http://{DEFAULT_COLLECTOR_ADDR}"),
        data_dir: None,
        last_error,
    }
}

fn loopback_is_listening(addr: &SocketAddr) -> bool {
    TcpStream::connect_timeout(addr, Duration::from_millis(250)).is_ok()
}

fn path_arg(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn config_paths_from_app(app: &AppHandle) -> Result<ConfigPaths, String> {
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Unable to resolve app config directory: {error}"))?;
    let app_local_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|error| format!("Unable to resolve app local data directory: {error}"))?;
    Ok(ConfigPaths::new(config_dir, app_local_data_dir))
}

fn load_config_view(app: &AppHandle) -> Result<DesktopConfigView, String> {
    let paths = config_paths_from_app(app)?;
    let store = DesktopConfigStore::new(paths.clone());
    let config = store.load_or_default()?;
    let secret_status =
        DpapiSecretStore::new(paths.secret_dir()).status(SecretKey::AiProviderApiKey)?;
    Ok(DesktopConfigView {
        config_path: store.paths().config_file(),
        config,
        ai_secret_status: secret_status,
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn collector_launch_args_use_loopback_and_user_data_paths() {
        let config = CollectorLaunchConfig::new(
            PathBuf::from("C:/Users/example/AppData/Local/TimeStateRecorder/data"),
            "127.0.0.1:4317".parse().unwrap(),
            1000,
        );

        assert_eq!(
            config.args(),
            vec![
                "serve",
                "--db",
                "C:/Users/example/AppData/Local/TimeStateRecorder/data/local.sqlite3",
                "--addr",
                "127.0.0.1:4317",
                "--poll-ms",
                "1000",
                "--blocker-config",
                "C:/Users/example/AppData/Local/TimeStateRecorder/data/blocker_config.json",
            ]
        );
        assert_eq!(config.api_url(), "http://127.0.0.1:4317");
    }
}
