# Troubleshooting

Common problems and how to fix them. Article-level errors are stored on the
article itself (`status: failed` + error message) — open the article to read
the exact message.

## Generation

### "No Anthropic API key set (Settings → API key, or $ANTHROPIC_API_KEY)"

No key is stored and the environment variable isn't set. Add a key under
**Settings → API keys**, or launch with `ANTHROPIC_API_KEY` exported.

### `Anthropic API 401 …`

The key is invalid or revoked. Generate a new key in the
[Anthropic console](https://console.anthropic.com/settings/keys) and save it
in Settings. (Saving a non-empty key overwrites the old one.)

### `Anthropic API 404 …` / model not found

The model id in **Settings → model** doesn't exist for your account. Reset it
to `claude-sonnet-5` or a snapshot id you have access to.

### "model did not return valid JSON"

The model's reply couldn't be parsed into `title`/`excerpt`/`body`. Parsing
already tolerates code fences and surrounding prose, so this is rare — retry
with **Generate now**; if it persists, try a different/newer model.

## Images

### Articles have no cover image

Checklist:

1. **Image enabled** is on for the site.
2. An OpenAI key is set (**Settings**) or `$OPENAI_API_KEY` is exported.
3. Check the terminal output — image failures are logged
   (`image: site X article Y: …`) but never block the article itself.

Use **Regenerate image** on the article after fixing the cause.

### `OpenAI images 400 …` mentioning `size`

Your chosen image model doesn't support the auto-selected size. Use
`gpt-image-1` (default) or a `dall-e-*` model — sizes are picked per model
family.

## Publishing

### `publish 401: …` / `publish 403: …`

The bearer token doesn't match what the Laravel site expects. Compare the
site's **Bearer token** field with the token configured on the server
(e.g. `BLOGWRITER_TOKEN` in the site's `.env`).

### `publish 404: …`

`base_url` + `api_path` doesn't resolve to your route. Verify the route is
registered under the API prefix (a route defined in `routes/api.php` as
`/articles` is served at `/api/articles`).

### `publish 405: …` or "too many redirects"

Usually an http→https or trailing-slash redirect chain. BlogWriter follows
redirects as POST (up to 5 hops), but it's best to configure the exact final
URL — use `https://` and the canonical host directly.

### `publish 422: …`

Laravel validation rejected the payload. Compare your validation rules with
the payload contract in [laravel-endpoint.md](laravel-endpoint.md) — note
that `image` can be a large data-URL string (allow long strings) and that
`category` is sent alongside `theme`.

### `request failed: …` (connection errors)

DNS/TLS/network-level failure — the site is unreachable from your machine.
Check the base URL and that the site is up.

## Scheduler

### Nothing generates automatically

1. The site must be **Active**.
2. Cadence must not be `manual`.
3. `next_run_at` must be in the past — check with:
   ```bash
   sqlite3 blogwriter.db 'SELECT name, cadence, active, datetime(next_run_at, "unixepoch") FROM sites;'
   ```
4. The app must be running (the scheduler ticks every 30 s while open).

### An article generated but wasn't published

Auto-publish is off for that site, or the publish failed — check the
article's status. Drafts can be published manually; failed articles show the
error and can be re-published.

## Build

### `rata codegen` / bindings out of date

After changing any `#[command]` signature, regenerate:

```bash
rata codegen && cd app && npm run build
```

### Cargo can't find the `elyra` crate

The `elyra` dependency is a **local path** in `Cargo.toml` (and
`@elyra/runtime` in `app/package.json`). Check out the Elyra framework and
point both paths at your local copy.

## Still stuck?

Open an issue with the exact error message (strip API keys and tokens!):
https://github.com/kwhorne/BlogWriter/issues
