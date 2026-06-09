use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "config.json";
const DEFAULT_RETENTION_DAYS: u32 = 30;
const DEFAULT_POLL_MS: u64 = 1000;
const DEFAULT_SCREENSHOT_INTERVAL_SECS: u64 = 60;
const DEFAULT_IDLE_THRESHOLD_SECS: u64 = 120;
const DEFAULT_API_PORT: u16 = 4317;
const DEFAULT_AI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_MAX_COMPLETION_TOKENS: u32 = 200_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigPaths {
    config_dir: PathBuf,
    app_local_data_dir: PathBuf,
}

impl ConfigPaths {
    pub fn new(config_dir: PathBuf, app_local_data_dir: PathBuf) -> Self {
        Self {
            config_dir,
            app_local_data_dir,
        }
    }

    pub fn config_file(&self) -> PathBuf {
        self.config_dir.join(CONFIG_FILE_NAME)
    }

    pub fn default_data_dir(&self) -> PathBuf {
        self.app_local_data_dir.join("data")
    }

    pub fn secret_dir(&self) -> PathBuf {
        self.config_dir.join("secrets")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopConfig {
    pub schema_version: u32,
    pub storage: StorageConfig,
    pub capture: CaptureConfig,
    pub privacy: PrivacyConfig,
    pub ai: AiProviderConfig,
    pub system: SystemConfig,
}

impl DesktopConfig {
    pub fn default_for_paths(paths: &ConfigPaths) -> Self {
        let data_dir = paths.default_data_dir();
        let mut config = Self {
            schema_version: 1,
            storage: StorageConfig {
                data_dir: PathBuf::new(),
                database_path: PathBuf::new(),
                screenshot_dir: PathBuf::new(),
                high_res_screenshot_dir: PathBuf::new(),
                retention_days: DEFAULT_RETENTION_DAYS,
            },
            capture: CaptureConfig {
                poll_ms: DEFAULT_POLL_MS,
                screenshot_interval_secs: DEFAULT_SCREENSHOT_INTERVAL_SECS,
                high_res_capture_enabled: true,
                input_capture_enabled: true,
                idle_threshold_secs: DEFAULT_IDLE_THRESHOLD_SECS,
            },
            privacy: PrivacyConfig {
                default_privacy_mode: PrivacyMode::Redacted,
                blocker_config_path: PathBuf::new(),
                external_ai_warning_accepted: false,
            },
            ai: AiProviderConfig {
                enabled: false,
                provider_preset: AiProviderPreset::CustomOpenAiCompatible,
                display_name: "Custom OpenAI-compatible provider".into(),
                base_url: String::new(),
                model: DEFAULT_AI_MODEL.into(),
                max_completion_tokens: DEFAULT_MAX_COMPLETION_TOKENS,
                vision_enabled: true,
                pipelines: AiPipelineConfig {
                    visual_analysis: false,
                    insight_reports: false,
                    daily_brief: false,
                },
            },
            system: SystemConfig {
                api_port: DEFAULT_API_PORT,
                launch_on_startup: false,
                start_minimized: false,
                tray_enabled: true,
            },
        };
        config.apply_data_dir(data_dir);
        config
    }

    pub fn apply_data_dir(&mut self, data_dir: PathBuf) {
        self.storage.data_dir = data_dir.clone();
        self.storage.database_path = data_dir.join("local.sqlite3");
        self.storage.screenshot_dir = data_dir.join("screenshots");
        self.storage.high_res_screenshot_dir = data_dir.join("high-res-screenshots");
        self.privacy.blocker_config_path = data_dir.join("blocker_config.json");
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub screenshot_dir: PathBuf,
    pub high_res_screenshot_dir: PathBuf,
    pub retention_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureConfig {
    pub poll_ms: u64,
    pub screenshot_interval_secs: u64,
    pub high_res_capture_enabled: bool,
    pub input_capture_enabled: bool,
    pub idle_threshold_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrivacyConfig {
    pub default_privacy_mode: PrivacyMode,
    pub blocker_config_path: PathBuf,
    pub external_ai_warning_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PrivacyMode {
    Redacted,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderConfig {
    pub enabled: bool,
    pub provider_preset: AiProviderPreset,
    pub display_name: String,
    pub base_url: String,
    pub model: String,
    pub max_completion_tokens: u32,
    pub vision_enabled: bool,
    pub pipelines: AiPipelineConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiProviderPreset {
    OpenAi,
    MiniMax,
    CustomOpenAiCompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiPipelineConfig {
    pub visual_analysis: bool,
    pub insight_reports: bool,
    pub daily_brief: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub api_port: u16,
    pub launch_on_startup: bool,
    pub start_minimized: bool,
    pub tray_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopConfigView {
    pub config_path: PathBuf,
    pub first_run: bool,
    pub config: DesktopConfig,
    pub ai_secret_status: SecretStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretStatus {
    pub present: bool,
    pub masked: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DesktopConfigStore {
    paths: ConfigPaths,
}

impl DesktopConfigStore {
    pub fn new(paths: ConfigPaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &ConfigPaths {
        &self.paths
    }

    pub fn is_first_run(&self) -> bool {
        !self.paths.config_file().exists()
    }

    pub fn load_or_default(&self) -> Result<DesktopConfig, String> {
        let path = self.paths.config_file();
        if !path.exists() {
            return Ok(DesktopConfig::default_for_paths(&self.paths));
        }

        let raw = fs::read_to_string(&path)
            .map_err(|error| format!("Unable to read desktop config: {error}"))?;
        serde_json::from_str(&raw)
            .map_err(|error| format!("Unable to parse desktop config: {error}"))
    }

    pub fn save(&self, config: &DesktopConfig) -> Result<(), String> {
        validate_config(config)?;
        fs::create_dir_all(&self.paths.config_dir)
            .map_err(|error| format!("Unable to create config directory: {error}"))?;
        fs::create_dir_all(&config.storage.data_dir)
            .map_err(|error| format!("Unable to create data directory: {error}"))?;

        let raw = serde_json::to_string_pretty(config)
            .map_err(|error| format!("Unable to serialize desktop config: {error}"))?;
        fs::write(self.paths.config_file(), raw)
            .map_err(|error| format!("Unable to write desktop config: {error}"))
    }
}

#[derive(Debug, Clone)]
pub struct ProviderTestInput {
    pub config: DesktopConfig,
    pub secret_status: SecretStatus,
}

impl ProviderTestInput {
    pub fn build_result(&self) -> Result<ProviderTestResult, String> {
        if !self.config.ai.enabled {
            return Ok(ProviderTestResult {
                status: "local_only".into(),
                request_kind: "none".into(),
                endpoint: None,
                model: None,
                uses_screenshots: false,
                secret: self.secret_status.clone(),
                message: "External AI is disabled.".into(),
            });
        }
        if self.config.ai.base_url.trim().is_empty() {
            return Err("AI provider base URL is required before testing.".into());
        }
        if self.config.ai.model.trim().is_empty() {
            return Err("AI provider model is required before testing.".into());
        }
        if !self.secret_status.present {
            return Err("AI provider API key is not configured.".into());
        }

        Ok(ProviderTestResult {
            status: "ready".into(),
            request_kind: "openai_compatible_text_chat_completions".into(),
            endpoint: Some(chat_completions_endpoint(&self.config.ai.base_url)),
            model: Some(self.config.ai.model.clone()),
            uses_screenshots: false,
            secret: self.secret_status.clone(),
            message: "Provider test is configured for a text-only chat completions request.".into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderTestResult {
    pub status: String,
    pub request_kind: String,
    pub endpoint: Option<String>,
    pub model: Option<String>,
    pub uses_screenshots: bool,
    pub secret: SecretStatus,
    pub message: String,
}

fn validate_config(config: &DesktopConfig) -> Result<(), String> {
    if config.storage.retention_days == 0 {
        return Err("Storage retention must be at least one day.".into());
    }
    if config.capture.poll_ms < 100 {
        return Err("Capture poll interval must be at least 100 ms.".into());
    }
    if config.system.api_port == 0 {
        return Err("API port must be non-zero.".into());
    }
    if config.ai.max_completion_tokens == 0 {
        return Err("AI max completion tokens must be non-zero.".into());
    }
    reject_path_traversal("data directory", &config.storage.data_dir)?;
    reject_path_traversal("database path", &config.storage.database_path)?;
    reject_path_traversal("screenshot directory", &config.storage.screenshot_dir)?;
    reject_path_traversal(
        "high-res screenshot directory",
        &config.storage.high_res_screenshot_dir,
    )?;
    Ok(())
}

fn reject_path_traversal(label: &str, path: &Path) -> Result<(), String> {
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir | std::path::Component::CurDir
        )
    }) {
        return Err(format!(
            "{label} cannot contain relative traversal segments."
        ));
    }
    Ok(())
}

fn chat_completions_endpoint(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, time::SystemTime};

    use super::{
        AiProviderPreset, CaptureConfig, ConfigPaths, DesktopConfig, DesktopConfigStore,
        PrivacyMode, ProviderTestInput, SecretStatus, StorageConfig, SystemConfig,
    };

    fn temp_root(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tsr-desktop-{name}-{nanos}"))
    }

    #[test]
    fn defaults_are_local_only_and_cover_all_phase4_categories() {
        let paths = ConfigPaths::new(temp_root("config"), temp_root("data"));
        let config = DesktopConfig::default_for_paths(&paths);

        assert_eq!(
            config.storage,
            StorageConfig {
                data_dir: paths.default_data_dir(),
                database_path: paths.default_data_dir().join("local.sqlite3"),
                screenshot_dir: paths.default_data_dir().join("screenshots"),
                high_res_screenshot_dir: paths.default_data_dir().join("high-res-screenshots"),
                retention_days: 30,
            }
        );
        assert_eq!(
            config.capture,
            CaptureConfig {
                poll_ms: 1000,
                screenshot_interval_secs: 60,
                high_res_capture_enabled: true,
                input_capture_enabled: true,
                idle_threshold_secs: 120,
            }
        );
        assert_eq!(config.privacy.default_privacy_mode, PrivacyMode::Redacted);
        assert!(!config.privacy.external_ai_warning_accepted);
        assert!(!config.ai.enabled);
        assert_eq!(
            config.ai.provider_preset,
            AiProviderPreset::CustomOpenAiCompatible
        );
        assert!(config.ai.base_url.is_empty());
        assert_eq!(config.ai.model, "gpt-4o-mini");
        assert_eq!(config.ai.max_completion_tokens, 200_000);
        assert_eq!(
            config.system,
            SystemConfig {
                api_port: 4317,
                launch_on_startup: false,
                start_minimized: false,
                tray_enabled: true,
            }
        );
    }

    #[test]
    fn config_store_round_trips_without_api_key_material() {
        let paths = ConfigPaths::new(temp_root("config"), temp_root("data"));
        let store = DesktopConfigStore::new(paths.clone());
        assert!(store.is_first_run());
        let mut config = store.load_or_default().unwrap();

        config.ai.enabled = true;
        config.ai.provider_preset = AiProviderPreset::MiniMax;
        config.ai.base_url = "https://api.minimax.example/v1".into();
        config.ai.model = "MiniMax-M3".into();
        store.save(&config).unwrap();

        let loaded = store.load_or_default().unwrap();
        assert_eq!(loaded, config);
        assert!(!store.is_first_run());

        let raw = fs::read_to_string(paths.config_file()).unwrap();
        assert!(raw.contains("MiniMax-M3"));
        assert!(!raw.contains("sk-live-secret"));
        assert!(!raw.to_lowercase().contains("api_key"));
        assert!(!raw.to_lowercase().contains("apikey"));
    }

    #[test]
    fn data_directory_selection_updates_related_storage_paths() {
        let paths = ConfigPaths::new(temp_root("config"), temp_root("data"));
        let mut config = DesktopConfig::default_for_paths(&paths);
        let selected = PathBuf::from("D:/TimeRecorderData");

        config.apply_data_dir(selected.clone());

        assert_eq!(config.storage.data_dir, selected);
        assert_eq!(
            config.storage.database_path,
            PathBuf::from("D:/TimeRecorderData/local.sqlite3")
        );
        assert_eq!(
            config.storage.screenshot_dir,
            PathBuf::from("D:/TimeRecorderData/screenshots")
        );
        assert_eq!(
            config.storage.high_res_screenshot_dir,
            PathBuf::from("D:/TimeRecorderData/high-res-screenshots")
        );
        assert_eq!(
            config.privacy.blocker_config_path,
            PathBuf::from("D:/TimeRecorderData/blocker_config.json")
        );
    }

    #[test]
    fn provider_test_plan_is_text_only_and_sanitized() {
        let paths = ConfigPaths::new(temp_root("config"), temp_root("data"));
        let mut config = DesktopConfig::default_for_paths(&paths);
        config.ai.enabled = true;
        config.ai.provider_preset = AiProviderPreset::MiniMax;
        config.ai.base_url = "https://api.minimax.example/v1".into();
        config.ai.model = "MiniMax-M3".into();

        let result = ProviderTestInput {
            config,
            secret_status: SecretStatus {
                present: true,
                masked: Some("••••CRET".into()),
            },
        }
        .build_result()
        .unwrap();

        assert_eq!(result.status, "ready");
        assert_eq!(
            result.request_kind,
            "openai_compatible_text_chat_completions"
        );
        assert!(!result.uses_screenshots);
        assert!(!format!("{result:?}").contains("sk-live-secret"));
    }
}
