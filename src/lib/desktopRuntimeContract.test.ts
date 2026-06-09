import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoText(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

describe("desktop runtime controls contract", () => {
  test("React runtime client exposes collector status, pause, and resume commands", () => {
    const runtimeTs = readRepoText("src/lib/desktopRuntime.ts");

    expect(runtimeTs).toContain("getCollectorStatus");
    expect(runtimeTs).toContain("startCollector");
    expect(runtimeTs).toContain("stopCollector");
    expect(runtimeTs).toContain("collector_status");
    expect(runtimeTs).toContain("start_collector");
    expect(runtimeTs).toContain("stop_collector");
  });

  test("Tauri main process wires tray actions for normal desktop controls", () => {
    const mainRs = readRepoText("src-tauri/src/main.rs");

    expect(mainRs).toContain("TrayIconBuilder");
    expect(mainRs).toContain("Pause Capture");
    expect(mainRs).toContain("Resume Capture");
    expect(mainRs).toContain("Open Settings");
    expect(mainRs).toContain("Generate Daily Brief");
    expect(mainRs).toContain("Quit");
    expect(mainRs).toContain("menu_event");
  });
});
