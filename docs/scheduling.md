# Scheduling

BlogWriter runs a background scheduler that generates (and optionally
publishes) articles for each active site on its own cadence.

## The scheduler loop

A background task starts when the app boots and ticks **every 30 seconds**:

1. Query all sites where `active = true` and `next_run_at <= now`.
2. Skip sites whose cadence is `manual`.
3. For each due site: generate an article (see the pipeline below), then
   advance `next_run_at` by the site's cadence interval.

A generation failure does **not** stop the schedule — the error is logged,
and `next_run_at` still advances, so a temporarily broken site doesn't retry
in a tight loop.

## Cadences

| Cadence | Interval |
|---|---|
| `hourly` | 3 600 s (1 hour) |
| `daily` | 86 400 s (24 hours) |
| `weekly` | 604 800 s (7 days) |
| `monthly` | 2 592 000 s (30 days) |
| `manual` | never auto-runs |

- The next run is measured **from the completion of the current run**
  (`now + interval`), not from the previously scheduled time — so runs drift
  slightly rather than accumulating a backlog.
- Unknown cadence strings fall back to `daily`.
- **New sites are due immediately**: `next_run_at` is set to "now" on
  creation, so the first article generates on the next tick.

## Theme rotation

A site's **Themes** field is a comma-separated list of topics. For each new
article, BlogWriter picks the theme at index `article_count % theme_count`,
i.e. a simple round-robin over the list based on how many articles the site
already has. With themes `A, B, C` you get `A, B, C, A, B, …`.

If the list is empty, the theme falls back to `general updates`.

The chosen theme also becomes the article's default **category** (editable
afterwards).

## The generate → publish pipeline

For a due site (or when you click **Generate now**):

```
pick theme ─▶ Claude writes article ─▶ store as draft (SQLite)
                                          │
                          image enabled?  ├─▶ OpenAI cover image (best-effort)
                                          │
                          auto-publish?   └─▶ POST to Laravel endpoint
                                                ├─ 2xx → status: published
                                                └─ err → status: failed (+ error msg)
```

Details:

- The article is **stored first** as a `draft` — a later image or publish
  failure never loses the text.
- Image generation is **best-effort**: if it fails, the article proceeds
  without a cover and the error is logged.
- Publishing sets `status` to `published` (and records `remote_id` +
  `published_at`) on success, or `failed` with the error message on failure.
  Failed articles can be re-published from the UI at any time.

## Manual controls

- **Generate now** — runs the pipeline for one site immediately, regardless
  of cadence (works for `manual` sites too). It does *not* advance
  `next_run_at`.
- **Publish** — (re)publishes an existing article, e.g. after editing a
  draft or fixing a failed endpoint.
- **Regenerate image** — replaces just the cover image of an existing
  article.
