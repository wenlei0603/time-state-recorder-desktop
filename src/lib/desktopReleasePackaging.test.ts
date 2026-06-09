import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function readRepoText(relativePath: string): string {
  return readFileSync(path.join(repoRoot, relativePath), "utf8");
}

describe("desktop release packaging contract", () => {
  test("package.json exposes a desktop release command", () => {
    const packageJson = JSON.parse(readRepoText("package.json"));

    expect(packageJson.scripts["desktop:release"]).toBe(
      "powershell -NoProfile -ExecutionPolicy Bypass -File scripts/package-release.ps1"
    );
  });

  test("release packager bundles the Tauri installer with checksum and manifest", () => {
    const script = readRepoText("scripts/package-release.ps1");

    expect(script).toContain("npm run desktop:build");
    expect(script).toContain("Time State Recorder Desktop_$Version_x64-setup.exe");
    expect(script).toContain("time-state-recorder-desktop-v$Version-windows-x64-setup.exe");
    expect(script).toContain("[System.Security.Cryptography.SHA256]");
    expect(script).toContain("ComputeHash");
    expect(script).toContain(".sha256");
    expect(script).toContain("release-manifest.json");
    expect(script).toContain("RELEASE_NOTES.md");
    expect(script).not.toContain("Start Time State Recorder.bat");
    expect(script).not.toContain("time-state-recorder-v$Version-windows-x64.zip");
  });

  test("tracked release notes and README document the local release command", () => {
    expect(existsSync(path.join(repoRoot, "docs/releases/v1.4.0.md"))).toBe(true);
    expect(readRepoText("README.md")).toContain("npm run desktop:release");
  });
});
