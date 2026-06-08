# Public Repo Boundary

This repository is intended to be safe to publish. Before each import, release,
or pull request, verify that no private runtime material is tracked.

## Must Never Be Tracked

- `data/`
- `logs/`
- `.env.local`
- `.launcher/`
- `.worktrees/`
- `.claude/`
- `.playwright-mcp/`
- `.superpowers/`
- SQLite files such as `*.sqlite3`, `*.db`, `*.db-wal`, and `*.db-shm`
- Screenshot directories and image evidence from a real user session
- Private Notion archive JSON or page identifiers
- Provider API keys or bearer tokens

## Required Checks

```powershell
git status --short
git ls-files
git ls-files | Select-String -Pattern '^(data|logs|output|reports|screenshots|high-res-screenshots|\\.launcher|\\.worktrees|worktrees)/|(^|/)(\\.claude|\\.playwright-mcp|\\.superpowers)(/|$)|\\.env\\.local$|\\.sqlite3$|\\.db$|\\.db-wal$|\\.db-shm$'
git grep -n -I -E 'api[_-]?key|bearer |sk-[A-Za-z0-9]|MINIMAX_API_KEY|OPENAI_API_KEY|secret' -- .
```

The pattern scan can produce false positives in docs and example config. Review
each hit before release.
