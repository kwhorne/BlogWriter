# BlogWriter Documentation

BlogWriter is a desktop app that generates blog articles with Anthropic Claude,
optionally illustrates them with OpenAI image models, and publishes them to
registered Laravel sites on a schedule.

## Contents

| Guide | Description |
|---|---|
| [Getting started](getting-started.md) | Prerequisites, building, running, and your first article |
| [Configuration](configuration.md) | App settings, environment variables, and every site option |
| [Scheduling](scheduling.md) | Cadences, the background scheduler, and theme rotation |
| [Image generation](images.md) | Cover images: providers, styles, aspect ratios, and optimization |
| [Laravel endpoint](laravel-endpoint.md) | The publish contract + a copy-paste route/controller for your site |
| [Auto-update](auto-update.md) | Signed in-app updates from GitHub Releases |
| [Architecture](architecture.md) | How the pieces fit together: modules, data flow, events |
| [Database](database.md) | SQLite schema, migrations, and where your data lives |
| [Troubleshooting](troubleshooting.md) | Common errors and how to fix them |

## Quick orientation

```
┌──────────────── BlogWriter (desktop) ────────────────┐      ┌── Laravel site ──┐
│                                                      │      │                  │
│  Scheduler ──▶ Claude (text) ──▶ SQLite ──▶ Publisher┼─────▶│ POST /api/…      │
│      ▲         OpenAI (image) ──────┘                │      │ (bearer token)   │
│      └── every 30s: run sites that are due          │      │                  │
└──────────────────────────────────────────────────────┘      └──────────────────┘
```

- **Everything is local.** Sites, articles, and settings live in a SQLite
  database next to the crate. No accounts, no cloud backend.
- **Two AI providers.** Claude writes the articles; OpenAI (optional) draws
  the cover images.
- **Your site owns publishing.** BlogWriter POSTs a JSON payload to an
  endpoint you control — see [laravel-endpoint.md](laravel-endpoint.md).

## Contributing to the docs

Docs live in this folder as plain Markdown. Corrections and improvements are
welcome — see [CONTRIBUTING.md](../CONTRIBUTING.md).
