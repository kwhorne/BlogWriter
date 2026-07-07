# Architecture

BlogWriter is an [Elyra](https://github.com/elyra) desktop app: a Rust backend
with an embedded Svelte 5 frontend, talking over a typed command bridge.

## High-level view

```
┌─────────────────────────── Rust backend ────────────────────────────┐
│                                                                     │
│  main.rs        boot: migrate DB, register providers, open window   │
│  handlers.rs    #[command] functions exposed to the frontend        │
│  schedule.rs    cadence math, generate→publish pipeline, scheduler  │
│  ai.rs          Anthropic Messages API (article text)               │
│  image.rs       OpenAI Images API + crop/optimize (cover images)    │
│  publish.rs     HTTP POST to the site's Laravel endpoint            │
│  models.rs      Site / Article / Setting (Elyra models over SQLite) │
│                                                                     │
└───────────────┬────────────────────────────────▲────────────────────┘
                │ typed commands (bindings.ts)   │ EventBus ("sites", "articles")
┌───────────────▼────────────────────────────────┴────────────────────┐
│                        Svelte 5 frontend (app/)                     │
│  App.svelte — sidebar of sites, article list, editors, settings     │
└─────────────────────────────────────────────────────────────────────┘
```

## Modules

### `main.rs` — boot

- Connects to `sqlite://<crate dir>/blogwriter.db` and runs all migrations
  on startup (skipped in codegen mode).
- Embeds the built frontend (`app/dist`) into the binary via `rust-embed`.
- Registers the `Scheduler` provider and the command handlers, then opens
  the window (with a tray icon).

### `handlers.rs` — the command surface

Commands are typed end-to-end: `rata codegen` generates
`app/src/bindings.ts` from the `#[command]` signatures.

| Command | Purpose |
|---|---|
| `list_sites` / `get_site` / `save_site` / `delete_site` | Site CRUD (delete cascades to the site's articles) |
| `list_articles` | Articles for a site, newest first |
| `generate_now` | Run the generate→publish pipeline for one site immediately |
| `publish_article` | (Re)publish an existing article |
| `update_article` | Edit title/excerpt/body/theme/category (slug, status, and image are kept) |
| `regenerate_image` | Replace just the cover image |
| `delete_article` | Delete one article |
| `get_settings` / `save_settings` | API keys + model choices (keys are write-only: the UI only learns *whether* a key is set) |

### `schedule.rs` — the pipeline

The heart of the app; see [scheduling.md](scheduling.md). Key design points:

- Articles are stored as drafts **before** image generation or publishing, so
  a downstream failure never loses text.
- The scheduler is a `tokio` task ticking every 30 s; each site tracks its own
  `next_run_at` (unix seconds) in the database.
- Failures advance the schedule anyway — no tight retry loops.

### `ai.rs` — text generation

- Calls the Anthropic Messages API directly (`reqwest`), no SDK.
- The prompt includes the site's description (if set), theme, tone, and
  language, and asks for **minified JSON** with `title` / `excerpt` / `body`.
- Parsing is tolerant: it extracts the outermost `{...}` from the reply, so
  stray prose or code fences around the JSON don't break it. Non-text blocks
  (e.g. thinking) in the response are skipped.

### `image.rs` — cover images

See [images.md](images.md). Handles both inline-base64 (`gpt-image-1`) and
URL-based (`dall-e-*`) responses, then center-crops, downscales, and
re-encodes to a JPEG data URL.

### `publish.rs` — delivery

- POSTs the article JSON to `{base_url}{api_path}` with an optional bearer
  token (see [laravel-endpoint.md](laravel-endpoint.md) for the payload).
- Follows redirects **manually as POST** (up to 5 hops) — reqwest's default
  policy would downgrade POST→GET on 301/302, causing spurious 405s.
- Extracts a `remote_id` from `{"id": …}` or `{"data": {"id": …}}` responses,
  numeric or string, best-effort.

## Events

The backend emits two events on the Elyra `EventBus`:

| Event | Payload | Emitted when |
|---|---|---|
| `sites` | site id | A site is created, updated, deleted, or its schedule advances |
| `articles` | site id | An article is created, updated, published, or deleted |

The frontend subscribes to these to refresh lists live — including for
articles generated in the background by the scheduler.

## Data

All state is in a single SQLite database; see [database.md](database.md).

## Frontend

- Svelte 5 + Vite, in `app/`. Markdown preview uses `marked`.
- Built output (`app/dist`) is embedded into the Rust binary — the packaged
  app is a single executable with no external asset files.
- After changing any `#[command]` signature, re-run `rata codegen` and commit
  the regenerated `app/src/bindings.ts`.
