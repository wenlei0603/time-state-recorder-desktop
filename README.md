# Time State Recorder Desktop

Time State Recorder Desktop is a local-first Windows workday memory app. It
will package the Time State Recorder collector and review UI into an
installable desktop product with first-run setup, user-selected local storage,
OpenAI-compatible AI provider configuration, and visible privacy controls.

This repository is the new public product boundary. It starts with product
docs, release hygiene, and a clean repo structure before source code is
imported from the current Time State Recorder product line.

## Product Direction

- Windows desktop app, initially built around Tauri v2.
- Local capture and local storage by default.
- Optional external AI analysis through an explicit OpenAI-compatible provider.
- MiniMax is supported as a provider preset, not a hard-coded dependency.
- User chooses where SQLite and screenshot evidence are stored.
- API keys are stored through a native secret-store abstraction, not in normal
  config files.
- Notion writes stay outside the desktop app in the first public version.

## Current Status

Status: Phase 4 config and secret-store foundation.

Implemented in this repo:

- Public repo hygiene files.
- Product PRD and goal contract.
- Safe example configuration.
- Security and contribution notes.
- Sanitized import of the current Rust collector, React review UI, local API
  docs, runbooks, and launcher scripts from the Time State Recorder product
  baseline.
- Tauri v2 desktop shell configuration with a primary Windows desktop window
  and minimal capability boundary.
- Desktop-managed collector sidecar wiring: the app prepares a Tauri sidecar
  binary and starts `tsr-collector` on loopback when the default API port is not
  already occupied.
- Typed desktop configuration for storage, capture, privacy, AI provider, and
  system settings.
- DPAPI-backed Windows secret storage for the AI provider key; the key is not
  serialized into `config.json`.

Not wired yet:

- First-run setup and Settings UI.
- Release signing, checksums, and GitHub release automation.

## Repository Layout

```text
docs/product/       Product requirements and execution contract
docs/security/      Security and privacy notes
examples/config/    Safe example config files with no secrets
scripts/            Future build, verification, and packaging helpers
collector/          Imported Rust collector and local API
src/                Imported React review UI
src-tauri/          Tauri v2 Windows desktop shell
```

The first source import intentionally keeps the existing source layout so the
baseline remains testable before desktop restructuring. Planned desktop layout:

```text
apps/desktop/       Tauri shell and React UI
crates/collector/   Rust collector and local API
crates/config/      Config and secret-store abstractions
```

## Privacy Boundary

The intended product stance is:

> Local-first by default, explicit when external analysis or archive tools are
> used.

The public repository must not contain user runtime state, local screenshots,
SQLite databases, logs, `.env.local`, API keys, private Notion artifacts, or
agent worktree directories.

## Next Steps

1. Import sanitized source from the current Time State Recorder `origin/master`
   product baseline.
2. Add desktop build foundation.
3. Replace env-file provider configuration with typed config and secret storage.
4. Build first-run setup and Settings UI.
5. Package a Windows release artifact.

## Desktop Development

Prerequisites:

- Node.js and npm.
- Rust/Cargo available on `PATH`.
- WebView2 runtime on Windows.
- Tauri-compatible Windows build tooling. This repo has been verified with the
  pinned `stable-x86_64-pc-windows-gnullvm` toolchain in `rust-toolchain.toml`;
  the standard Tauri recommendation is Rust via rustup plus Visual Studio Build
  Tools/MSVC.

```powershell
npm run desktop:info
npm run desktop:prepare-sidecar
npm run desktop:dev
npm run desktop:build
```

The current desktop shell reuses the imported Vite/React review UI. Collector
startup is managed as a Tauri sidecar when `127.0.0.1:4317` is free; if another
collector is already listening there, the desktop state reports it as an
external collector instead of stopping it. Database location selection and AI
provider credentials now have backend APIs; first-run and Settings UI are the
next product layer.

`npm run desktop:build` creates the Windows installer under
`target/x86_64-pc-windows-gnullvm/release/bundle/nsis/` for the current pinned
toolchain.

## Configuration

Non-secret settings live in the app config directory as `config.json`. The
schema covers storage paths, capture intervals, privacy defaults,
OpenAI-compatible AI provider metadata, and desktop system behavior. See
[examples/config/config.example.json](examples/config/config.example.json).

AI provider keys are stored separately through a Windows DPAPI-backed secret
store under the app config directory. Settings code should use the Tauri
commands `set_ai_provider_api_key` and `clear_ai_provider_api_key`; never write
keys into `config.json`, logs, screenshots, tests, or examples.
