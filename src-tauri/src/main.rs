use std::{
    env,
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    sync::Mutex,
    time::Duration,
};

use tauri::{
    App, AppHandle, Emitter, Manager, RunEvent, State,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_dialog::DialogExt;
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
const TRAY_OPEN_APP: &str = "open_app";
const TRAY_OPEN_SETTINGS: &str = "open_settings";
const TRAY_DAILY_BRIEF: &str = "generate_daily_brief";
const TRAY_PAUSE_CAPTURE: &str = "pause_capture";
const TRAY_RESUME_CAPTURE: &str = "resume_capture";
const TRAY_QUIT: &str = "quit";

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
fn collector_status(
    app: AppHandle,
    state: State<'_, CollectorState>,
) -> Result<CollectorDesktopStatus, String> {
    Ok(refresh_collector_status(&app, &state)?)
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
    sync_launch_on_startup(&app, config.system.launch_on_startup)?;
    load_config_view(&app)
}

#[tauri::command]
fn choose_data_directory(app: AppHandle) -> Result<Option<String>, String> {
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| {
            path.into_path()
                .map(|path| path_arg(&path))
                .map_err(|error| format!("Unable to resolve selected folder: {error}"))
        })
        .transpose()
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let handle = app.handle().clone();
            let startup_config = load_desktop_config(&handle);
            let tray_enabled = startup_config
                .as_ref()
                .map(|config| config.system.tray_enabled)
                .unwrap_or(true);
            configure_tray(app, tray_enabled)?;
            let state = app.state::<CollectorState>();
            if let Ok(config) = startup_config.as_ref() {
                apply_start_minimized(app, config);
                if let Err(error) = sync_launch_on_startup(&handle, config.system.launch_on_startup)
                {
                    state.set_error(default_collector_status(Some(error)));
                }
            }
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
            choose_data_directory,
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

fn configure_tray(app: &mut App, enabled: bool) -> tauri::Result<()> {
    if !enabled {
        return Ok(());
    }

    let open_app = MenuItem::with_id(app, TRAY_OPEN_APP, "Open App", true, None::<&str>)?;
    let open_settings =
        MenuItem::with_id(app, TRAY_OPEN_SETTINGS, "Open Settings", true, None::<&str>)?;
    let daily_brief = MenuItem::with_id(
        app,
        TRAY_DAILY_BRIEF,
        "Generate Daily Brief",
        true,
        None::<&str>,
    )?;
    let pause_capture =
        MenuItem::with_id(app, TRAY_PAUSE_CAPTURE, "Pause Capture", true, None::<&str>)?;
    let resume_capture = MenuItem::with_id(
        app,
        TRAY_RESUME_CAPTURE,
        "Resume Capture",
        true,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, TRAY_QUIT, "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &open_app,
            &open_settings,
            &daily_brief,
            &separator,
            &pause_capture,
            &resume_capture,
            &separator,
            &quit,
        ],
    )?;

    let mut tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("Time State Recorder")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, menu_event| {
            match menu_event.id().0.as_str() {
                TRAY_OPEN_APP => show_main_window(app),
                TRAY_OPEN_SETTINGS => {
                    show_main_window(app);
                    let _ = app.emit("tsr://open-settings", ());
                }
                TRAY_DAILY_BRIEF => {
                    show_main_window(app);
                    let _ = app.emit("tsr://open-daily-brief", ());
                }
                TRAY_PAUSE_CAPTURE => {
                    let state = app.state::<CollectorState>();
                    if state.snapshot().managed {
                        let _ = state.stop();
                    }
                }
                TRAY_RESUME_CAPTURE => {
                    let state = app.state::<CollectorState>();
                    let _ = start_collector_from_app(app, &state);
                }
                TRAY_QUIT => app.exit(0),
                _ => {}
            }
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn apply_start_minimized(app: &App, config: &DesktopConfig) {
    if !config.system.tray_enabled || !should_start_minimized(config, env::args()) {
        return;
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn should_start_minimized<I>(config: &DesktopConfig, args: I) -> bool
where
    I: IntoIterator<Item = String>,
{
    config.system.start_minimized
        || args
            .into_iter()
            .any(|arg| arg == "--minimized" || arg == "--start-minimized")
}

fn sync_launch_on_startup(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager
            .enable()
            .map_err(|error| format!("Unable to enable launch on startup: {error}"))
    } else {
        manager
            .disable()
            .map_err(|error| format!("Unable to disable launch on startup: {error}"))
    }
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
            last_error: Some(port_conflict_message(config)),
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

    fn set_stopped_after_external_port_release(
        &self,
        config: &CollectorLaunchConfig,
    ) -> CollectorDesktopStatus {
        let status = CollectorDesktopStatus {
            status: "stopped".into(),
            managed: false,
            pid: None,
            api_url: config.api_url(),
            data_dir: Some(config.data_dir()),
            last_error: Some(
                "The configured API port is free now. Click Resume Capture to start the desktop-managed collector."
                    .into(),
            ),
        };
        if let Ok(mut runtime) = self.inner.lock() {
            runtime.status = status.clone();
            runtime.child = None;
        }
        status
    }
}

fn refresh_collector_status(
    app: &AppHandle,
    state: &CollectorState,
) -> Result<CollectorDesktopStatus, String> {
    let snapshot = state.snapshot();
    if snapshot.status != "external" {
        return Ok(snapshot);
    }

    let config = CollectorLaunchConfig::from_app(app)?;
    if loopback_is_listening(&config.addr) {
        Ok(state.set_external(&config))
    } else {
        Ok(state.set_stopped_after_external_port_release(&config))
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

fn port_conflict_message(config: &CollectorLaunchConfig) -> String {
    format!(
        "Port conflict: {} is already in use by another process. Quit that process or change API Port in Settings, then click Resume Capture.",
        config.addr
    )
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

fn load_desktop_config(app: &AppHandle) -> Result<DesktopConfig, String> {
    let paths = config_paths_from_app(app)?;
    DesktopConfigStore::new(paths).load_or_default()
}

fn load_config_view(app: &AppHandle) -> Result<DesktopConfigView, String> {
    let paths = config_paths_from_app(app)?;
    let store = DesktopConfigStore::new(paths.clone());
    let first_run = store.is_first_run();
    let config = store.load_or_default()?;
    let secret_status =
        DpapiSecretStore::new(paths.secret_dir()).status(SecretKey::AiProviderApiKey)?;
    Ok(DesktopConfigView {
        config_path: store.paths().config_file(),
        first_run,
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

    #[test]
    fn external_status_explains_port_conflict_recovery() {
        let config = CollectorLaunchConfig::new(
            PathBuf::from("D:/TSR/data"),
            "127.0.0.1:5317".parse().unwrap(),
            1000,
        );
        let state = CollectorState::default();

        let status = state.set_external(&config);

        assert_eq!(status.status, "external");
        assert!(!status.managed);
        let message = status.last_error.unwrap();
        assert!(message.contains("Port conflict"));
        assert!(message.contains("change API Port in Settings"));
        assert!(message.contains("Resume Capture"));
    }

    #[test]
    fn start_minimized_decision_uses_config_or_launch_flag() {
        let paths = ConfigPaths::new(temp_path("config"), temp_path("data"));
        let mut config = DesktopConfig::default_for_paths(&paths);

        assert!(!should_start_minimized(
            &config,
            ["time-state-recorder-desktop".to_string()].into_iter()
        ));
        assert!(should_start_minimized(
            &config,
            [
                "time-state-recorder-desktop".to_string(),
                "--minimized".to_string()
            ]
            .into_iter()
        ));

        config.system.start_minimized = true;
        assert!(should_start_minimized(
            &config,
            ["time-state-recorder-desktop".to_string()].into_iter()
        ));
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tsr-desktop-main-test-{name}"))
    }
}
