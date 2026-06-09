# Time State Recorder Desktop Goal Contract

Date: 2026-06-09

Objective: turn Time State Recorder into a clean open-source Windows desktop
product, publish it to a new repo, and keep execution sequential with explicit
evidence gates.

## 1. Contract Scope

This contract covers:

- PRD approval.
- New public repo creation.
- Sanitized code migration from the current Time State Recorder product line.
- Windows desktop shell.
- First-run configuration.
- AI provider settings for OpenAI-compatible backends.
- User-controlled local storage path.
- Privacy and capture controls.
- Packaging and public release readiness.

This contract does not cover:

- Direct Notion writes from the desktop app.
- Cloud account service.
- Team monitoring or organization admin workflows.
- Merging the separate Dayflow clone project.

## 2. Source of Truth

Current planning source:

- Workspace: `D:\CodexInfra\docs\projects\time-state-recorder`
- Current checked-out branch: `master`
- Current checked-out commit: `3123f67`
- Newer remote product baseline: `origin/master` at `fe42aab`

Execution source rule:

- Use `origin/master` as the functional migration baseline unless the user
  explicitly selects a different branch.
- Keep the old repo as the development source until the new repo exists.
- Once the new repo is created and verified, use the new repo as the execution
  source for desktop product work.

## 3. Global Invariants

- Do not stop the existing live `4317` backend unless the user explicitly asks
  for a live restart or deployment.
- Do not copy private runtime state into the public repo.
- Do not expose API keys in logs, config files, tests, docs, screenshots, or git
  history.
- Do not describe optional external AI mode as fully local.
- Keep MiniMax as a provider preset behind an OpenAI-compatible provider model.
- Keep Notion writes outside the desktop app for the first public version.
- Keep Dayflow clone and Time State Recorder separate.

## 4. Sequential Phases

### Phase 0: PRD Review Gate

Goal:

- Confirm that `docs/product/windows-desktop-open-source-prd.md` is the product
  baseline.

Inputs:

- PRD document.
- User review.

Done evidence:

- User approves the PRD or requests specific edits.
- If edits are requested, the PRD is updated and re-reviewed.

Stop condition:

- Do not create the new public repo before this gate is accepted unless the user
  explicitly asks to proceed without review.

### Phase 1: New Repo Boundary

Goal:

- Create a clean repo boundary for `time-state-recorder-desktop`.

Tasks:

- Choose local path and GitHub repo name.
- Create repo with a clean initial commit.
- Add `.gitignore` covering runtime data, logs, env files, local worktrees, and
  agent artifacts.
- Add license, README shell, security note, and contribution note.

Done evidence:

- `git status --short` in the new repo is clean after initial commit.
- `git remote -v` points to the intended new repo when GitHub creation is done.
- No `data/`, `logs/`, `.env.local`, `.launcher/`, `.worktrees/`, `.claude/`,
  `.playwright-mcp/`, or `.superpowers/` directories are tracked.

### Phase 2: Sanitized Code Import

Goal:

- Import the current product code into the new repo without private state.

Tasks:

- Import from `origin/master`.
- Preserve collector, WebUI, scripts, tests, docs, and sample fixtures that are
  safe for public release.
- Reorganize into the PRD repo structure or document a deliberate simpler
  structure for the first cut.
- Convert private `.env.local` behavior into example config files.

Done evidence:

- `git ls-files` in the new repo shows only intended source, docs, examples, and
  scripts.
- Pattern scan finds no obvious API keys, private Notion page IDs, local diary
  artifacts, private screenshots, or live SQLite database files.
- Existing baseline tests still run or documented failures are tied to the
  migration step being executed next.

### Phase 3: Desktop Foundation

Goal:

- Add a Windows desktop shell around the current React UI and Rust collector.

Tasks:

- Add Tauri v2 desktop app.
- Load the existing React UI inside the desktop window.
- Start or supervise the collector from the desktop app.
- Preserve local API contracts at first.
- Add desktop Health view fields for API URL, collector PID, data directory,
  and app version.

Done evidence:

- Desktop app opens without manually starting a browser.
- Collector starts from the desktop app on a loopback address.
- Today view loads live or sample data inside the native window.
- Manual quit shuts down the desktop-managed collector cleanly.

### Phase 4: Config and Secret Store

Goal:

- Replace env-file product configuration with typed config and secret storage.

Tasks:

- Add config schema for storage, capture, privacy, AI provider, and system
  settings.
- Add `SecretStore` abstraction.
- Add Windows implementation using DPAPI, Windows Credential Manager, or an
  equivalent user-scoped Windows secret backend.
- Add config read/write APIs for desktop UI.
- Add provider test command that avoids user screenshots.

Done evidence:

- Tests prove config defaults and config save/load behavior.
- Tests or manual verification prove API key values are not written to
  `config.json`.
- UI can set, mask, replace, and remove an API key.
- Logs do not print the secret value during provider test.

### Phase 5: First-Run and Settings UI

Goal:

- Make setup possible from UI only.

Tasks:

- Add first-run wizard.
- Add storage location picker.
- Add capture/privacy controls.
- Add AI provider settings page.
- Add local-only mode.
- Add visible external-AI warning before enabling provider-backed analysis.

Done evidence:

- Clean config directory opens onboarding.
- User can complete onboarding without editing files.
- User can choose data directory and start capture.
- User can configure MiniMax or custom OpenAI-compatible provider through UI.
- User can return to Settings and change storage, capture, privacy, and AI
  provider settings.

### Phase 6: Desktop Controls

Goal:

- Make the app behave like a normal Windows desktop product.

Tasks:

- Add tray menu.
- Add pause/resume capture.
- Add open app, settings, generate Daily Brief, and quit actions.
- Add start-minimized and launch-on-startup options.
- Add robust port conflict handling.

Done evidence:

- Tray menu can pause and resume capture.
- Main window reflects capture state.
- Quit leaves no desktop-managed collector process behind.
- Port conflict produces a visible error and recovery path.

### Phase 7: Public Packaging

Goal:

- Produce a public Windows release artifact.

Tasks:

- Build Windows x64 installer or setup executable.
- Keep portable ZIP as secondary artifact if useful.
- Add example configs and privacy/security docs.
- Add release notes and checksums.
- Add GitHub Actions or documented local release command.

Done evidence:

- Release artifact installs or launches on Windows.
- First-run setup works from a clean user profile or clean config directory.
- README explains Local only mode and External AI mode.
- Checksum is produced for the release artifact.

### Phase 8: Publish

Goal:

- Publish the new repo and first release.

Tasks:

- Push new repo to GitHub.
- Create release tag.
- Upload release artifact and checksum.
- Verify GitHub page displays README and release correctly.

Done evidence:

- New GitHub repo URL exists and is reachable.
- Release URL exists and contains the Windows artifact.
- Fresh clone passes documented verification commands or clearly documented
  prerequisites are the only blockers.

## 5. Verification Matrix

Required checks before claiming the full goal is complete:

- PRD exists and matches the agreed product direction.
- Goal contract exists and phases are satisfied in order.
- New repo exists.
- Public repo has no private runtime data or secrets.
- Desktop app can be launched without browser scripts.
- Settings UI supports AI provider configuration.
- Settings UI supports user-selected data directory.
- API key is stored outside normal config files.
- Local-only mode works without external AI credentials.
- Capture pause/resume works.
- Existing Today, reports, Daily Brief, evidence, and health workflows remain
  reachable.
- Windows release artifact exists.
- Verification commands and manual smoke checks have current output evidence.

## 6. Current Execution Status

Current repo:

- Local path: `D:\CodexInfra\docs\projects\time-state-recorder-desktop`
- GitHub repo: `https://github.com/wenlei0603/time-state-recorder-desktop`

Completed evidence checkpoints:

- Phase 1 repo boundary is established.
- Phase 2 sanitized source import is in the new repo.
- Phase 3 Tauri desktop foundation and sidecar supervision are implemented.
- Phase 4 typed config and DPAPI-backed secret storage are implemented.
- Phase 5 first-run Settings UI, data-directory picker, local-only mode, and
  OpenAI-compatible provider settings are implemented.
- Phase 6 tray controls, pause/resume capture, launch-on-startup,
  start-minimized behavior, and port-conflict recovery are implemented.

Current next action:

- Complete Phase 7 public packaging, then Phase 8 GitHub release publication.
