# Contributing

This project is in early repo-boundary setup. The first contribution priority is
to keep the public repository clean, reproducible, and safe to clone.

## Ground Rules

- Do not commit runtime data, screenshots, SQLite databases, logs, or API keys.
- Do not commit `.env.local` or machine-specific config.
- Keep MiniMax, OpenAI, and custom gateways behind the OpenAI-compatible
  provider abstraction.
- Keep Notion writes outside the desktop app unless the product contract is
  changed.
- Keep Dayflow clone work separate from Time State Recorder Desktop.

## Expected Checks

The full verification suite will be added when source code is imported. Planned
checks:

```powershell
cargo fmt --all
cargo test
npm test
npm run build
npm run desktop:build
```

Before opening a pull request, also run a secret scan or equivalent pattern scan
for provider keys, private Notion IDs, local screenshot paths, and SQLite files.
