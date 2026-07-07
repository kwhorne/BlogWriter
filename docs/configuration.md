# Configuration

BlogWriter has two configuration layers: **app settings** (API keys and model
choices, shared by all sites) and **per-site options**.

## App settings (Settings screen)

Stored in the `settings` table of the local database.

| Setting | Key | Default | Description |
|---|---|---|---|
| Anthropic API key | `anthropic_api_key` | — | Used for article generation |
| Text model | `anthropic_model` | `claude-sonnet-5` | Any Anthropic Messages API model id; pin a snapshot (e.g. `claude-sonnet-5-YYYYMMDD`) for reproducibility |
| OpenAI API key | `openai_api_key` | — | Used for cover images only |
| Image model | `image_model` | `gpt-image-1` | `gpt-image-1` or a `dall-e-*` model |

Notes:

- Leaving a key field **empty** on save keeps the previously stored key
  unchanged (so you can update the model without re-entering keys).
- The Settings screen never displays stored keys — only whether one is set.

## Environment variables

Environment variables act as **fallbacks** when no key is stored in settings:

| Variable | Used when |
|---|---|
| `ANTHROPIC_API_KEY` | `anthropic_api_key` setting is empty |
| `OPENAI_API_KEY` | `openai_api_key` setting is empty |

A key stored in Settings always wins over the environment.

## Per-site options

| Field | Default | Description |
|---|---|---|
| **Name** | — | Display name in the sidebar |
| **Description** | *(empty)* | What the site is about — audience, purpose, voice. Injected into the generation prompt so articles fit the site's focus. Strongly recommended. |
| **Base URL** | — | e.g. `https://blog.example.com` (trailing slash is fine) |
| **API path** | `/api/articles` | Appended to the base URL for publishing |
| **Bearer token** | *(empty)* | Sent as `Authorization: Bearer <token>`; omitted if empty |
| **Themes** | *(empty)* | Comma-separated topics. BlogWriter rotates through them round-robin (see [scheduling.md](scheduling.md)). Empty falls back to "general updates". |
| **Tone** | `informative and friendly` | Free text, passed to the model |
| **Language** | `English` | The language articles are written in |
| **Cadence** | `daily` | `hourly` \| `daily` \| `weekly` \| `monthly` \| `manual` |
| **Active** | on | Inactive sites are skipped by the scheduler |
| **Auto-publish** | on | Publish immediately after generating; off = leave as `draft` |
| **Image enabled** | on | Generate a cover image per article (requires OpenAI key) |
| **Image style** | `flat isometric` | Free-text illustration style, e.g. `risograph grain` |
| **Image format** | `16:9` | Aspect ratio as `W:H`, e.g. `16:9`, `1:1`, `4:5` |

### Choosing a cadence

- `manual` sites **never** auto-generate — use **Generate now**.
- Any unrecognized cadence value falls back to `daily`.
- See [scheduling.md](scheduling.md) for exactly when the next run happens.

## Where configuration lives

Everything — keys, sites, articles — is stored in `blogwriter.db` (SQLite)
next to the crate. Treat this file as **secret material**: it contains your
API keys and site tokens. It is gitignored; never commit or share it.

See [database.md](database.md) for the schema.
