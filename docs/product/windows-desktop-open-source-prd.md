# Time State Recorder Desktop PRD

Date: 2026-06-09

Target repo: `time-state-recorder-desktop`

Status: Draft for review

## 1. Product Summary

Time State Recorder Desktop packages the current local-first Windows workday
memory layer into an installable desktop application. The desktop app should
let a normal user install the product, choose where local data is stored,
configure an OpenAI-compatible AI backend, review privacy settings, start or
pause capture, and open the daily review UI without running scripts or opening a
separate browser.

The product remains local-first by default. External AI analysis is explicit,
provider-configured, and visible in the UI. MiniMax is treated as one
OpenAI-compatible provider preset, not as a hard-coded product dependency.

## 2. Current Baseline

This PRD uses `origin/master` at `fe42aab` as the product baseline because the
checked-out local `master` is still `v1.0.0` at `3123f67`, while the remote
baseline already contains the newer activity-insights product line.

Current baseline capabilities:

- Rust Windows collector with local SQLite storage.
- React WebUI for Today Flow, dashboard, timeline, activity review, screenshot
  evidence, input activity, collector health, 5-hour reports, and Daily Brief.
- Local REST API on `127.0.0.1:4317`.
- Optional model-backed visual analysis, insight reports, and Daily Brief.
- MiniMax-backed implementation through environment variables.
- Read-only Notion daily archive payload API.
- Windows ZIP packaging with scripts, not a native desktop app.

## 3. Problem

The project currently works for a developer who can run scripts, edit env
files, understand local ports, and keep private runtime state separate from
source. That is not enough for an open-source product.

An open-source Windows user needs:

- A normal desktop entrypoint.
- First-run setup instead of `.env.local` editing.
- Safe storage for API keys.
- A UI for AI provider and model configuration.
- A UI for database and evidence storage location.
- Visible privacy controls before capture starts.
- Clear indication when data may leave the machine.
- An installable build that does not bundle private data, logs, keys, or local
  Notion workflows.

## 4. Product Goals

1. Ship Time State Recorder as an installable Windows desktop app.
2. Preserve the local-first architecture and current review workflows.
3. Replace env-file AI configuration with a settings UI and secret store.
4. Let the user choose, inspect, and change the local data directory.
5. Make capture state, AI state, storage state, and privacy state visible.
6. Publish a clean public repo with build instructions, example configs, and no
   private runtime artifacts.

## 5. Non-Goals

- No employee monitoring workflow.
- No cloud account system.
- No team sync, billing, or time-sheet workflow.
- No direct Notion writes from the desktop app in the first public version.
- No guarantee that optional AI analysis is private once enabled.
- No migration of the separate Dayflow clone project into this repo.
- No public release containing existing `data/`, `logs/`, `.env.local`, local
  Notion helper scripts, or private archive artifacts.

## 6. Target Users

Primary users:

- Individual Windows knowledge workers.
- Researchers, developers, writers, analysts, and operators who need a
  reviewable memory of a workday.
- Users with their own OpenAI-compatible model provider, including MiniMax or a
  self-hosted gateway.

Secondary users:

- Developers who want to inspect, fork, or extend a local-first activity
  recorder.
- Agent builders who want a local API for daily activity context.

## 7. Product Principles

- Evidence before summary.
- Local capture before external integration.
- Explicit external AI boundary.
- User-controlled storage path.
- Reviewable interpretation, not productivity scoring.
- Default to redacted UI modes before raw evidence display.
- Public repo must be safe to clone and inspect.

## 8. Recommended Desktop Architecture

Recommended approach: Tauri v2 shell around the existing React UI and Rust
collector.

Reasons:

- The project already uses Rust and React.
- A Tauri shell can provide a native window, tray menu, filesystem dialogs,
  startup integration, sidecar management, and installer packaging without
  replacing the current UI.
- The current local API can be preserved at first, reducing migration risk.
- Tauri's Windows distribution path supports MSI or setup executable packaging
  through WiX or NSIS.

Alternative 1: Electron plus Rust sidecar.

- Pros: mature desktop ecosystem, strong updater story, easy Node integration.
- Cons: heavier runtime, a separate Node main-process security surface, and less
  alignment with the existing Rust collector.

Alternative 2: keep the current ZIP plus launcher scripts.

- Pros: fastest path from current state.
- Cons: not a true desktop app, poor first-run configuration, no native secret
  store, weak product perception.

Decision for this PRD: build Tauri v2 first, keep Electron as fallback only if
Tauri packaging or sidecar control blocks the Windows release.

## 9. Runtime Architecture

Initial desktop runtime:

```text
Tauri desktop process
  -> starts or supervises tsr-collector
  -> loads React review UI in a native WebView
  -> talks to collector through local loopback API
  -> reads/writes non-secret config
  -> stores AI API keys through a secret-store abstraction
  -> exposes tray actions and first-run setup
```

Collector API:

- Bind to `127.0.0.1` only.
- Prefer an app-managed port, defaulting to `4317` when free.
- Surface actual API URL in Settings and Health.
- Keep current API contracts for Today Flow, reports, Daily Brief, analysis
  status, screenshots, input activity, and Notion archive payload.

Future runtime option:

- Move selected control-plane calls from HTTP to Tauri commands or IPC after the
  desktop product is stable.

## 10. Configuration Model

Non-secret settings are stored in a user-scoped config file. Proposed default:

```text
%LOCALAPPDATA%\TimeStateRecorder\config.json
```

Default data directory:

```text
%LOCALAPPDATA%\TimeStateRecorder\data\
```

Default data files:

```text
%LOCALAPPDATA%\TimeStateRecorder\data\local.sqlite3
%LOCALAPPDATA%\TimeStateRecorder\data\screenshots\
%LOCALAPPDATA%\TimeStateRecorder\data\high-res-screenshots\
```

Config categories:

- Storage: database path, screenshot directory, retention days.
- Capture: poll interval, screenshot interval, idle threshold, high-res capture
  toggle, input capture toggle.
- Privacy: raw/redacted default, blocker rules path, evidence display warnings.
- AI provider: provider preset, base URL, model name, max completion tokens,
  vision support, enabled pipelines.
- System: API port, launch on startup, start minimized, tray behavior, update
  channel.

Secret values:

- API keys are never stored in `config.json`.
- API keys are stored through `SecretStore`.
- UI shows only provider name, model name, key presence, and a masked key suffix.
- Logs must not print API keys, full request headers, or raw `.env` values.

## 11. AI Provider Requirements

The first public version supports one provider type:

```text
OpenAI-compatible chat completions
```

Provider presets:

- OpenAI
- MiniMax
- Custom OpenAI-compatible endpoint

Required fields:

- Display name
- Base URL
- API key
- Model
- Max completion tokens
- Vision/image input support

Pipeline toggles:

- Screenshot visual summaries
- 5-minute visual window summaries
- 5-hour insight reports
- Daily Brief

Required behavior:

- Local metadata analysis remains available without external AI.
- External AI requests are disabled until a provider is explicitly enabled.
- First-run setup explains that screenshots or structured summaries may be sent
  to the configured provider.
- Provider test sends a minimal non-sensitive request first.
- Vision test uses a bundled synthetic image fixture, not a user's screenshot.
- `finish_reason=length` responses are rejected instead of stored as complete
  reports.

## 12. First-Run Onboarding

The app opens into setup when no valid config exists.

Step 1: Welcome and privacy boundary

- Explain local capture.
- Explain optional external AI.
- Require the user to continue before capture starts.

Step 2: Storage location

- Show default path.
- Let user pick another directory.
- Validate write access.
- Show expected disk usage drivers: SQLite, thumbnails, high-res screenshots.

Step 3: Capture controls

- Enable or disable window tracking.
- Enable or disable screenshots.
- Enable or disable high-res screenshots.
- Enable or disable input capture.
- Link to blocker rules.

Step 4: AI service

- Choose Local only, OpenAI, MiniMax, or Custom.
- Enter base URL, model, and API key when external AI is chosen.
- Test provider.
- Select which analysis pipelines are allowed.

Step 5: Start

- Start collector.
- Show health checks.
- Open Today view.

## 13. Desktop UI Information Architecture

Primary shell:

- Left rail: Today, Reports, Evidence, Activity, Settings, Health.
- Top bar: capture status, AI status, data path indicator, pause button.
- Main panel: current selected view.
- Tray menu: Open, Pause Capture, Resume Capture, Generate Daily Brief, Settings,
  Quit.

Settings pages:

1. AI Services
   - Provider preset.
   - Base URL.
   - Model.
   - API key set/change/remove.
   - Test connection.
   - Pipeline toggles.
   - Last successful analysis status.

2. Storage
   - Current data directory.
   - Change storage location.
   - Open folder.
   - Retention policy.
   - Database size and screenshot size.

3. Capture and Privacy
   - Screenshot cadence.
   - High-res evidence toggle.
   - Idle threshold.
   - Input capture toggle.
   - Blocker rules editor or JSON import/export.
   - Default raw/redacted mode.

4. System
   - API address.
   - Launch on startup.
   - Start minimized.
   - Logs path.
   - Version and update channel.

5. Integrations
   - Notion archive endpoint preview.
   - Export JSON/Markdown.
   - No Notion token in v1.

## 14. Privacy and Security Requirements

Minimum requirements:

- Bind local API to loopback only.
- Do not enable permissive CORS.
- Store secrets outside config files.
- Never bundle `.env.local`.
- Never bundle user data, screenshots, logs, reports, or live SQLite files.
- Show a visible warning before raw text segments or screenshots are displayed.
- Provide pause/resume capture from the main window and tray.
- Provide a clear "Local only" mode.
- Preserve blocker rules before screenshot capture.

Secret-store implementation:

- Create a Rust `SecretStore` trait so the UI and collector do not depend on a
  specific backend.
- Windows implementation should use Windows user-scoped secret protection, such
  as DPAPI or Windows Credential Manager.
- If Tauri Stronghold is adopted in a future version, it remains behind the same
  `SecretStore` interface.

## 15. Open-Source Repo Boundary

New public repo must include:

- Source code.
- Sample config without secrets.
- Synthetic/sample data only.
- Build scripts.
- Tests.
- Product docs.
- Security and privacy docs.
- License.
- Contribution guide.

New public repo must exclude:

- `data/`
- `logs/`
- `.env.local`
- `.launcher/`
- `.worktrees/`
- `.claude/`
- `.playwright-mcp/`
- `.superpowers/`
- Output release zips from private runs.
- Notion Principles OS helper scripts unless explicitly sanitized and documented
  as optional external tooling.

Recommended repo structure:

```text
time-state-recorder-desktop/
  apps/desktop/          # Tauri shell and React UI
  crates/collector/      # Rust collector and local API
  crates/config/         # config and secret-store abstractions
  docs/product/          # PRD, goal contract, launch notes
  docs/security/         # privacy and secret handling docs
  examples/config/       # safe example config files
  scripts/               # build, verify, package helpers
```

## 16. Release Requirements

First public release should produce:

- Windows x64 installer or setup executable.
- Portable ZIP only as secondary artifact.
- README with "Local only" and "External AI" modes.
- Example provider configs with no keys.
- Screenshot-safe demo images.
- Release notes.
- SHA256 checksums.

Verification commands must cover:

- Rust formatting and tests.
- TypeScript tests.
- Web build.
- Desktop build.
- Secret scan or equivalent pattern scan for keys and private paths.
- First-run smoke test on a clean config directory.

## 17. Success Metrics

MVP product success:

- A clean Windows machine can install and open the app.
- A user can start local-only capture without editing files.
- A user can set an OpenAI-compatible backend and verify it from UI.
- A user can choose where the database and screenshots are stored.
- A user can pause capture from tray or main window.
- Daily review, 5-hour reports, and Daily Brief remain reachable.
- Public repo clone contains no private local state.

## 18. Risks

- Tauri packaging may require switching the Windows Rust target from the current
  GNU setup to MSVC or adding a separate desktop build path.
- Local WebView2 availability and installer size need a deliberate packaging
  choice.
- Sidecar collector management can create port conflicts or stale processes if
  shutdown is not robust.
- API key storage is high-risk; secrets must not pass through React local
  storage or config files.
- The product can overclaim privacy if docs do not clearly explain optional AI
  analysis.
- The existing live backend on `4317` should not be stopped during planning or
  migration unless the user explicitly asks for a live swap.

## 19. References Consulted

- Tauri Windows installer docs: https://v2.tauri.app/distribute/windows-installer/
- Electron autoUpdater docs for fallback comparison: https://www.electronjs.org/docs/latest/api/auto-updater
- Microsoft DPAPI `CryptProtectData`: https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata
- Microsoft Credential Management API: https://learn.microsoft.com/en-us/windows/win32/secauthn/credential-management-api
- Tauri Stronghold reference: https://tauri.app/fr/reference/javascript/stronghold/
