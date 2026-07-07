# Getting started

This guide takes you from a fresh clone to your first generated article.

## Prerequisites

| Requirement | Notes |
|---|---|
| [Rust](https://rustup.rs) | Stable toolchain, edition 2021 |
| [Node.js](https://nodejs.org) 20+ | For building the Svelte frontend |
| [Elyra framework](https://github.com/elyra) | Currently referenced by **local path** — see below |
| Anthropic API key | Required for article generation |
| OpenAI API key | Optional — only needed for cover images |

> **Path dependencies:** `Cargo.toml` references the `elyra` crate and
> `app/package.json` references `@elyra/runtime` by local filesystem path.
> Check out the Elyra framework locally and adjust both paths to match your
> machine before building.

## Build and run

```bash
git clone https://github.com/kwhorne/BlogWriter.git
cd BlogWriter

# 1. Build the frontend (embedded into the binary from app/dist)
cd app && npm install && npm run build && cd ..

# 2. Generate typed TS bindings for the command surface
rata codegen

# 3. Run the app
cargo run
```

On first launch the app creates `blogwriter.db` next to the crate and runs all
migrations automatically.

## First article in five steps

1. **Settings → API keys** — paste your Anthropic key. (Alternatively set the
   `ANTHROPIC_API_KEY` environment variable before launching.) The default
   model is `claude-sonnet-5`.
2. *(Optional)* Add an OpenAI key for cover images (`gpt-image-1` by default).
3. **+ New site** — fill in:
   - **Name** and **Description** (what the site is about — this steers the writing)
   - **Base URL** + **API path** + **Bearer token** (your Laravel endpoint,
     see [laravel-endpoint.md](laravel-endpoint.md))
   - **Themes** — comma-separated topics; BlogWriter rotates through them
   - **Tone**, **Language**, **Cadence**
4. Toggle **Auto-publish** if you want articles pushed immediately after
   generation; leave it off to review drafts first.
5. Click **Generate now** — or wait for the scheduler. New sites are due
   immediately, so the first article generates on the next scheduler tick
   (within ~30 seconds) if the cadence isn't `manual`.

## Verifying it worked

- The article appears in the site's article list with status `draft`,
  `published`, or `failed`.
- `failed` articles keep the error message from the publish attempt — open
  the article to read it, fix the cause, and hit **Publish** again.

## Running the tests

```bash
cargo test
```

Unit tests cover cadence math, slug generation, JSON parsing of model output,
image cropping/optimization, and database CRUD. Live API calls (Anthropic,
OpenAI, your Laravel endpoint) are **not** exercised in tests.

## Next steps

- [Configuration](configuration.md) — every setting explained
- [Laravel endpoint](laravel-endpoint.md) — set up the receiving site
- [Scheduling](scheduling.md) — how cadences and rotation work
