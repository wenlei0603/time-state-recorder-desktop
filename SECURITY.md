# Security Policy

Time State Recorder Desktop is intended to be local-first software for a single
user. It may store sensitive desktop activity, screenshot evidence, local text
segments, model outputs, and API provider credentials.

## Supported Versions

No public release is supported yet. Security reporting instructions will apply
after the first tagged release.

## Reporting a Vulnerability

Until a public maintainer contact is published, open a private security advisory
on GitHub after this repository is published. Do not include API keys, raw
screenshots, or private activity logs in public issues.

## Security Requirements

- Local API endpoints must bind to loopback by default.
- API keys must not be stored in `config.json`, localStorage, source files, test
  fixtures, logs, screenshots, or release artifacts.
- Public releases must not bundle `data/`, `logs/`, `.env.local`, local SQLite
  files, private screenshots, or private Notion archive artifacts.
- Optional external AI analysis must be visibly disclosed before it is enabled.
- "Local only" mode must remain available without external AI credentials.
- Raw screenshots and text segments require visible UI warnings.
