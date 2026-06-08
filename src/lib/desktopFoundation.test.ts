import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function repoPath(relativePath: string): string {
  return path.join(repoRoot, relativePath);
}

function readJson(relativePath: string): unknown {
  const absolutePath = repoPath(relativePath);
  expect(existsSync(absolutePath), `${relativePath} should exist`).toBe(true);
  return JSON.parse(readFileSync(absolutePath, "utf8"));
}

function readText(relativePath: string): string {
  const absolutePath = repoPath(relativePath);
  expect(existsSync(absolutePath), `${relativePath} should exist`).toBe(true);
  return readFileSync(absolutePath, "utf8");
}

describe("desktop foundation", () => {
  test("package scripts expose Tauri desktop commands", () => {
    const packageJson = readJson("package.json") as {
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
      scripts?: Record<string, string>;
    };

    expect(packageJson.dependencies?.["@tauri-apps/api"]).toMatch(/^2\./);
    expect(packageJson.devDependencies?.["@tauri-apps/cli"]).toMatch(/^2\./);
    expect(packageJson.scripts?.["desktop:dev"]).toBe("tauri dev");
    expect(packageJson.scripts?.["desktop:build"]).toBe("tauri build");
    expect(packageJson.scripts?.["desktop:info"]).toBe("tauri info");
    expect(packageJson.scripts?.["desktop:prepare-sidecar"]).toBe(
      "powershell -NoProfile -ExecutionPolicy Bypass -File scripts/prepare-tauri-sidecar.ps1"
    );
    expect(packageJson.scripts?.["desktop:dev-ui"]).toBe(
      "npm run desktop:prepare-sidecar && vite --host 127.0.0.1 --port 5173 --strictPort"
    );
  });

  test("Tauri config wraps the Vite app as the Windows desktop shell", () => {
    const config = readJson("src-tauri/tauri.conf.json") as {
      productName?: string;
      version?: string;
      identifier?: string;
      build?: Record<string, unknown>;
      app?: {
        security?: Record<string, unknown>;
        windows?: Array<Record<string, unknown>>;
      };
      bundle?: Record<string, unknown>;
    };

    expect(config.productName).toBe("Time State Recorder Desktop");
    expect(config.version).toBe("1.4.0");
    expect(config.identifier).toBe("io.github.wenlei0603.time-state-recorder-desktop");
    expect(config.build?.beforeDevCommand).toBe("npm run desktop:dev-ui");
    expect(config.build?.devUrl).toBe("http://127.0.0.1:5173");
    expect(config.build?.beforeBuildCommand).toBe(
      "npm run build && npm run desktop:prepare-sidecar"
    );
    expect(config.build?.frontendDist).toBe("../dist");
    expect(config.app?.security?.capabilities).toEqual(["main"]);
    expect(config.bundle?.active).toBe(true);
    expect(config.bundle?.targets).toEqual(["nsis"]);
    expect(config.bundle?.icon).toEqual(["icons/icon.ico"]);
    expect(config.bundle?.externalBin).toEqual(["bin/tsr-collector"]);
    expect(existsSync(repoPath("src-tauri/icons/icon.ico"))).toBe(true);

    const mainWindow = config.app?.windows?.[0];
    expect(mainWindow?.label).toBe("main");
    expect(mainWindow?.title).toBe("Time State Recorder");
    expect(mainWindow?.width).toBeGreaterThanOrEqual(1200);
    expect(mainWindow?.height).toBeGreaterThanOrEqual(760);
  });

  test("desktop crate is in the Rust workspace and exposes a health command", () => {
    expect(readText("Cargo.toml")).toContain('members = ["collector", "src-tauri"]');

    const tauriCargo = readText("src-tauri/Cargo.toml");
    expect(tauriCargo).toContain('name = "time-state-recorder-desktop"');
    expect(tauriCargo).toContain('tauri = { version = "2"');
    expect(tauriCargo).toContain('tauri-build = { version = "2"');
    expect(tauriCargo).toContain('tauri-plugin-shell = { version = "2"');

    const mainRs = readText("src-tauri/src/main.rs");
    expect(mainRs).toContain("#[tauri::command]");
    expect(mainRs).toContain("fn desktop_health(state: State<'_, CollectorState>) -> DesktopHealth");
    expect(mainRs).toContain("fn start_collector(");
    expect(mainRs).toContain("fn stop_collector(");
    expect(mainRs).toContain("CollectorLaunchConfig");
    expect(mainRs).toContain("tauri_plugin_shell::init()");
    expect(mainRs).toContain("RunEvent::ExitRequested");
    expect(mainRs).toContain("state.stop()");
    expect(mainRs).toContain("tauri::generate_handler![");
    expect(mainRs).toContain("desktop_health");
    expect(mainRs).toContain("collector_status");
    expect(mainRs).toContain("start_collector");
    expect(mainRs).toContain("stop_collector");
  });

  test("main window has an explicit minimal capability", () => {
    const capability = readJson("src-tauri/capabilities/main.json") as {
      identifier?: string;
      windows?: string[];
      permissions?: string[];
    };

    expect(capability.identifier).toBe("main");
    expect(capability.windows).toEqual(["main"]);
    expect(capability.permissions).toEqual(["core:default"]);
  });

  test("sidecar preparation script is checked in", () => {
    const script = readText("scripts/prepare-tauri-sidecar.ps1");

    expect(script).toContain("cargo build -p tsr-collector --release");
    expect(script).toContain("tsr-collector-$targetTriple.exe");
    expect(script).toContain("x86_64-pc-windows-msvc.exe");
    expect(script).toContain("src-tauri");
    expect(script).toContain("bin");
  });
});
