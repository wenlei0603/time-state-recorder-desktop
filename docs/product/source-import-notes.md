# Sanitized Source Import Notes

Date: 2026-06-09

Imported source baseline:

- Source repo: `D:\CodexInfra\docs\projects\time-state-recorder`
- Git ref: `origin/master`
- Commit: `fe42aab`

## Import Method

The import used `git archive origin/master` from the source repo and copied only
selected tracked paths into this new repo. This avoided current-worktree private
runtime state such as `data/`, `logs/`, `.env.local`, `.launcher/`, and local
agent worktrees.

## Imported

- `.cargo/`
- `collector/`
- `src/`
- `scripts/`
- `docs/api/`
- `docs/runbooks/`
- `docs/infra.md`
- `docs/product-pitch.md`
- Cargo, npm, TypeScript, Vite, and Windows launcher files

## Deliberately Not Imported

- `reports/`
- `docs/dayflow-engineering-knowledge/`
- `docs/superpowers/`
- `docs/bugs/`
- `docs/slides/`
- Root source README, because this repo has a desktop-product README
- Local runtime or agent directories from the source checkout

## Structure Decision

The first import keeps the existing root-level collector and WebUI layout. This
is deliberate: it gives Phase 2 a testable baseline before Phase 3 introduces a
Tauri desktop app and before later phases reorganize code into `apps/desktop/`
and `crates/`.
