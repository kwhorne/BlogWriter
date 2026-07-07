# BlogWriter

> AI-powered blog automation for Laravel sites — generate articles with Claude, illustrate them with GPT Image, and publish on a schedule.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-ff3e00.svg)](https://svelte.dev)

BlogWriter is a desktop app that generates blog articles with **Anthropic Claude**
and automatically publishes them to registered **Laravel** sites on a schedule
(hourly / daily / weekly / monthly). Optionally, it generates a matching header
image with **OpenAI's image models**. Built on the [Elyra](https://github.com/elyra)
framework (Rust backend + Svelte frontend).

## Features

- 🤖 **AI article generation** — Claude writes complete articles (title, excerpt, Markdown body) based on your themes, tone, and language
- 🖼️ **AI header images** — optional per-site image generation with configurable style and aspect ratio
- 🗓️ **Scheduling** — each site publishes on its own cadence: hourly, daily, weekly, monthly, or manual
- 🌐 **Multi-site** — manage any number of Laravel sites, each with its own themes, tone, language, and API token
- 🚀 **Auto-publish** — push articles straight to your site's API, or review and publish manually
- 💾 **Local-first** — everything lives in a local SQLite database; no cloud account required

## How it works

```
┌──────────── BlogWriter (desktop) ──────────────┐        ┌─── Laravel site ───┐
│  scheduler ──▶ Anthropic (text)  ──▶ SQLite    │        │ POST /api/articles │
│               OpenAI (image)  └──▶ publisher ──┼──────▶ │  (bearer token)    │
└─────────────────────────────────────────────────┘        └────────────────────┘
```

- **Sites & articles** live in a local SQLite database (Elyra models + migrations).
- **Generation** uses the Anthropic Messages API; the model returns JSON
  (`title` / `excerpt` / `body` in Markdown).
- **Publishing** POSTs the article to the site's Laravel endpoint — see
  [docs/laravel-endpoint.md](docs/laravel-endpoint.md) for the contract and a
  copy-paste route/controller.

📚 **Full documentation lives in [docs/](docs/README.md)** — getting started,
configuration, scheduling, image generation, architecture, database schema,
and troubleshooting.
- **Scheduling** is a background task; each site advances its own `next_run_at`
  by its cadence. Set cadence to `manual` to only generate on demand.

## Prerequisites

- [Rust](https://rustup.rs) (stable, edition 2021)
- [Node.js](https://nodejs.org) 20+
- Access to the [Elyra framework](https://github.com/kwhorne/elyra-framework)
  repository (the `elyra` crate is a git dependency fetched over SSH; the
  frontend's `@elyra/runtime` is still a local `file:` path in
  `app/package.json` — adjust it to your checkout)
- An **Anthropic API key** (required) and an **OpenAI API key** (optional, for
  header images)

## Getting started

```bash
git clone https://github.com/kwhorne/BlogWriter.git
cd BlogWriter

# frontend
cd app && npm install && npm run build && cd ..

# typed bindings + run
rata codegen
cargo run
```

Then in the app:

1. **Settings → API keys**: paste your Anthropic key (or set `$ANTHROPIC_API_KEY`),
   and optionally an OpenAI key for image generation.
2. **+ New site**: name, base URL, API path, bearer token, themes, tone,
   language, cadence. Toggle **Auto-publish** to push immediately after
   generating.
3. **Generate now** to produce an article on demand, or let the scheduler run.

The SQLite database is created as `blogwriter.db` next to the crate. It contains
your API keys and site tokens — **never commit it**.

## Laravel integration

Your site only needs a single authenticated endpoint that accepts the article
payload. A minimal, ready-to-paste route + controller (including image
handling) is documented in [docs/laravel-endpoint.md](docs/laravel-endpoint.md).

## Documentation

| Guide | Description |
|---|---|
| [Getting started](docs/getting-started.md) | Build, run, and generate your first article |
| [Configuration](docs/configuration.md) | Settings, env vars, and every site option |
| [Scheduling](docs/scheduling.md) | Cadences, the scheduler, and theme rotation |
| [Image generation](docs/images.md) | Cover images: models, styles, optimization |
| [Laravel endpoint](docs/laravel-endpoint.md) | The publish contract for your site |
| [Architecture](docs/architecture.md) | Modules, data flow, and events |
| [Database](docs/database.md) | SQLite schema and migrations |
| [Troubleshooting](docs/troubleshooting.md) | Common errors and fixes |

## Project layout

```
src/            Rust backend (Elyra app)
  ai.rs         Anthropic article generation
  image.rs      OpenAI header-image generation
  publish.rs    HTTP publisher for Laravel endpoints
  schedule.rs   Background scheduler / cadence math
  handlers.rs   Commands exposed to the frontend
  models.rs     SQLite models
app/            Svelte 5 frontend (Vite)
migrations/     SQLite migrations
docs/           Documentation (see docs/README.md)
```

## Testing

```bash
cargo test
```

Covered: cadence math, slug generation, model JSON parsing, and database
CRUD. The AI/publish HTTP paths are implemented but not exercised against live
services in CI — a valid Anthropic key and a reachable Laravel endpoint are
required to see them succeed end to end.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for
how to file issues and submit pull requests.

## Building releases

```bash
scripts/build-macos.sh   # bundle + Developer ID-signed .app, .dmg, .zip (arm64)
scripts/build-linux.sh   # Docker build → tar.gz (host arch)
```

Artifacts land in `dist/`. See the scripts for signing/notarization details.

## License

[MIT](LICENSE) © Knut Whorne
