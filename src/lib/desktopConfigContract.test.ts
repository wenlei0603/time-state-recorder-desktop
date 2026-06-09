import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoText(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

describe("desktop config and secret-store contract", () => {
  test("Tauri exposes config and AI-key commands for the settings UI", () => {
    const mainRs = readRepoText("src-tauri/src/main.rs");

    expect(mainRs).toContain("get_desktop_config");
    expect(mainRs).toContain("save_desktop_config");
    expect(mainRs).toContain("set_ai_provider_api_key");
    expect(mainRs).toContain("clear_ai_provider_api_key");
    expect(mainRs).toContain("choose_data_directory");
    expect(mainRs).toContain("test_ai_provider_connection");
    expect(mainRs).toContain("tauri::generate_handler![");
  });

  test("Rust config code separates non-secret config from secret storage", () => {
    const configRs = readRepoText("src-tauri/src/config.rs");
    const secretsRs = readRepoText("src-tauri/src/secrets.rs");

    expect(configRs).toContain("struct DesktopConfig");
    expect(configRs).toContain("first_run");
    expect(configRs).toContain("apply_data_dir");
    expect(configRs).toContain("struct StorageConfig");
    expect(configRs).toContain("struct CaptureConfig");
    expect(configRs).toContain("struct PrivacyConfig");
    expect(configRs).toContain("struct AiProviderConfig");
    expect(configRs).toContain("struct SystemConfig");
    expect(configRs).not.toContain("api_key:");
    expect(configRs).not.toContain("apiKey:");

    expect(secretsRs).toContain("trait SecretStore");
    expect(secretsRs).toContain("struct DpapiSecretStore");
    expect(secretsRs).toContain("CryptProtectData");
    expect(secretsRs).toContain("CryptUnprotectData");
  });

  test("example config matches the typed non-secret schema", () => {
    const example = JSON.parse(readRepoText("examples/config/config.example.json")) as {
      storage?: Record<string, unknown>;
      capture?: Record<string, unknown>;
      privacy?: Record<string, unknown>;
      ai?: Record<string, unknown>;
      system?: Record<string, unknown>;
    };
    const raw = JSON.stringify(example).toLowerCase();

    expect(example.storage?.dataDir).toBeDefined();
    expect(example.capture?.pollMs).toBe(1000);
    expect(example.privacy?.defaultPrivacyMode).toBe("redacted");
    expect(example.ai?.enabled).toBe(false);
    expect(example.ai?.providerPreset).toBe("customOpenAiCompatible");
    expect(example.system?.apiPort).toBe(4317);
    expect(raw).not.toContain("api_key");
    expect(raw).not.toContain("apikey");
    expect(raw).not.toContain("secretname");
  });
});
