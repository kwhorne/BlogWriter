# Database

BlogWriter stores everything in a single SQLite file, `blogwriter.db`,
created next to the crate on first run.

> ⚠️ **This file contains secrets** — your Anthropic/OpenAI API keys and the
> bearer tokens for every site. It is gitignored (`*.db`); never commit or
> share it.

## Migrations

Migrations live in `migrations/` as paired `NNNN_name.sql` /
`NNNN_name.down.sql` files and run **automatically on app startup** (tracked
in the `_elyra_migrations` table).

| Migration | Adds |
|---|---|
| `0001_init` | `sites`, `articles`, `settings` tables |
| `0002_images` | Per-site image options + `articles.image` |
| `0003_site_description` | `sites.description` (steers generation) |
| `0004_article_category` | `articles.category` (distinct from `theme`) |

## Tables

### `sites`

One row per registered Laravel site.

| Column | Type | Default | Description |
|---|---|---|---|
| `id` | INTEGER PK | auto | |
| `name` | TEXT | — | Display name |
| `description` | TEXT | `''` | What the site is about; injected into the generation prompt |
| `base_url` | TEXT | — | e.g. `https://blog.example.com` |
| `api_path` | TEXT | `/api/articles` | Publish endpoint path |
| `token` | TEXT | `''` | Bearer token (secret) |
| `themes` | TEXT | `''` | Comma-separated topics, rotated round-robin |
| `tone` | TEXT | `informative and friendly` | |
| `language` | TEXT | `English` | |
| `cadence` | TEXT | `daily` | `hourly` \| `daily` \| `weekly` \| `monthly` \| `manual` |
| `active` | INTEGER | `1` | Inactive sites are skipped by the scheduler |
| `auto_publish` | INTEGER | `1` | Publish right after generating |
| `image_enabled` | INTEGER | `1` | Generate a cover image per article |
| `image_style` | TEXT | `flat isometric` | Free-text illustration style |
| `image_format` | TEXT | `16:9` | Aspect ratio `W:H` |
| `next_run_at` | INTEGER | `0` | Unix seconds; when the next article is due |
| `created_at` / `updated_at` | INTEGER | `0` | Unix seconds, managed by the model layer |

### `articles`

One row per generated article. Deleting a site deletes its articles.

| Column | Type | Default | Description |
|---|---|---|---|
| `id` | INTEGER PK | auto | |
| `site_id` | INTEGER | — | FK to `sites` (indexed: `idx_articles_site`) |
| `title` | TEXT | — | |
| `slug` | TEXT | `''` | URL-friendly, derived from the title (ascii, max 80 chars) |
| `excerpt` | TEXT | `''` | 1–2 sentence summary |
| `body` | TEXT | `''` | Full article, **Markdown** |
| `theme` | TEXT | `''` | The topic it was generated for (drives rotation) |
| `category` | TEXT | `''` | Blog category; defaults to the theme, editable |
| `status` | TEXT | `draft` | `draft` \| `published` \| `failed` |
| `remote_id` | TEXT | `''` | Id returned by the Laravel site on publish |
| `error` | TEXT | `''` | Last publish error (when `status = failed`) |
| `image` | TEXT | `''` | Cover as a JPEG **data URL**, or empty |
| `published_at` | INTEGER | `0` | Unix seconds of the last successful publish |
| `created_at` / `updated_at` | INTEGER | `0` | |

### `settings`

Key/value app settings.

| Column | Type | Description |
|---|---|---|
| `id` | INTEGER PK | |
| `name` | TEXT | Setting key |
| `value` | TEXT | Setting value |
| `created_at` / `updated_at` | INTEGER | |

Known keys: `anthropic_api_key`, `anthropic_model`, `openai_api_key`,
`image_model` — see [configuration.md](configuration.md).

## Inspecting the database

```bash
sqlite3 blogwriter.db '.tables'
sqlite3 blogwriter.db 'SELECT id, name, cadence, next_run_at FROM sites;'
sqlite3 blogwriter.db 'SELECT id, title, status, published_at FROM articles ORDER BY id DESC LIMIT 10;'
```

## Backup / reset

- **Backup:** copy `blogwriter.db` while the app is closed.
- **Reset:** delete `blogwriter.db`; the app recreates it (empty) on next
  launch. You will need to re-enter API keys and sites.
