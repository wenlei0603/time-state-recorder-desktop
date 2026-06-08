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

Status: Phase 2 sanitized source import.

Implemented in this repo:

- Public repo hygiene files.
- Product PRD and goal contract.
- Safe example configuration.
- Security and contribution notes.
- Sanitized import of the current Rust collector, React review UI, local API
  docs, runbooks, and launcher scripts from the Time State Recorder product
  baseline.

Not imported yet:

- Tauri desktop shell.
- Build and release automation.

## Repository Layout

```text
docs/product/       Product requirements and execution contract
docs/security/      Security and privacy notes
examples/config/    Safe example config files with no secrets
scripts/            Future build, verification, and packaging helpers
collector/          Imported Rust collector and local API
src/                Imported React review UI
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
