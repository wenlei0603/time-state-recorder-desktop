import { invoke } from "@tauri-apps/api/core";

export type ProviderPreset = "openAi" | "miniMax" | "customOpenAiCompatible";
export type DesktopPrivacyMode = "redacted" | "raw";

export type StorageConfig = {
  dataDir: string;
  databasePath: string;
  screenshotDir: string;
  highResScreenshotDir: string;
  retentionDays: number;
};

export type CaptureConfig = {
  pollMs: number;
  screenshotIntervalSecs: number;
  highResCaptureEnabled: boolean;
  inputCaptureEnabled: boolean;
  idleThresholdSecs: number;
};

export type PrivacyConfig = {
  defaultPrivacyMode: DesktopPrivacyMode;
  blockerConfigPath: string;
  externalAiWarningAccepted: boolean;
};

export type AiPipelineConfig = {
  visualAnalysis: boolean;
  insightReports: boolean;
  dailyBrief: boolean;
};

export type AiProviderConfig = {
  enabled: boolean;
  providerPreset: ProviderPreset;
  displayName: string;
  baseUrl: string;
  model: string;
  maxCompletionTokens: number;
  visionEnabled: boolean;
  pipelines: AiPipelineConfig;
};

export type SystemConfig = {
  apiPort: number;
  launchOnStartup: boolean;
  startMinimized: boolean;
  trayEnabled: boolean;
};

export type DesktopConfig = {
  schemaVersion: number;
  storage: StorageConfig;
  capture: CaptureConfig;
  privacy: PrivacyConfig;
  ai: AiProviderConfig;
  system: SystemConfig;
};

export type SecretStatus = {
  present: boolean;
  masked?: string;
};

export type DesktopConfigView = {
  configPath: string;
  firstRun: boolean;
  config: DesktopConfig;
  aiSecretStatus: SecretStatus;
};

export type ProviderTestResult = {
  status: string;
  requestKind: string;
  endpoint?: string;
  model?: string;
  usesScreenshots: boolean;
  secret: SecretStatus;
  message: string;
};

export type DesktopConfigClient = {
  getConfig: () => Promise<DesktopConfigView>;
  saveConfig: (config: DesktopConfig) => Promise<DesktopConfigView>;
  setApiKey: (secret: string) => Promise<SecretStatus>;
  clearApiKey: () => Promise<SecretStatus>;
  chooseDataDirectory: () => Promise<string | null | undefined>;
  testProvider: () => Promise<ProviderTestResult>;
};

function invokeDesktop<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const hasTauriRuntime =
    typeof window !== "undefined" &&
    typeof (window as Window & { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__ === "object";
  if (!hasTauriRuntime) {
    return Promise.reject(
      new Error("Desktop settings are unavailable in browser preview.")
    );
  }
  return invoke<T>(command, args);
}

export const tauriDesktopConfigClient: DesktopConfigClient = {
  getConfig: () => invokeDesktop<DesktopConfigView>("get_desktop_config"),
  saveConfig: (config) =>
    invokeDesktop<DesktopConfigView>("save_desktop_config", { config }),
  setApiKey: (secret) =>
    invokeDesktop<SecretStatus>("set_ai_provider_api_key", { secret }),
  clearApiKey: () => invokeDesktop<SecretStatus>("clear_ai_provider_api_key"),
  chooseDataDirectory: () =>
    invokeDesktop<string | null>("choose_data_directory"),
  testProvider: () => invokeDesktop<ProviderTestResult>("test_ai_provider_connection")
};
