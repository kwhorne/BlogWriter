-- Registered Laravel sites that receive generated articles.
CREATE TABLE IF NOT EXISTS sites (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    base_url    TEXT NOT NULL,               -- e.g. https://blog.example.com
    api_path    TEXT NOT NULL DEFAULT '/api/articles',
    token       TEXT NOT NULL DEFAULT '',    -- bearer token for the Laravel endpoint
    themes      TEXT NOT NULL DEFAULT '',    -- comma-separated topics to write about
    tone        TEXT NOT NULL DEFAULT 'informative and friendly',
    language    TEXT NOT NULL DEFAULT 'English',
    cadence     TEXT NOT NULL DEFAULT 'daily', -- hourly|daily|weekly|monthly|manual
    active      INTEGER NOT NULL DEFAULT 1,
    auto_publish INTEGER NOT NULL DEFAULT 1,  -- publish immediately after generating
    next_run_at INTEGER NOT NULL DEFAULT 0,   -- unix seconds; when the next article is due
    created_at  INTEGER NOT NULL DEFAULT 0,
    updated_at  INTEGER NOT NULL DEFAULT 0
);

-- Generated articles and their publish status.
CREATE TABLE IF NOT EXISTS articles (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    site_id      INTEGER NOT NULL,
    title        TEXT NOT NULL,
    slug         TEXT NOT NULL DEFAULT '',
    excerpt      TEXT NOT NULL DEFAULT '',
    body         TEXT NOT NULL DEFAULT '',    -- markdown
    theme        TEXT NOT NULL DEFAULT '',
    status       TEXT NOT NULL DEFAULT 'draft', -- draft|published|failed
    remote_id    TEXT NOT NULL DEFAULT '',     -- id returned by the Laravel site
    error        TEXT NOT NULL DEFAULT '',
    created_at   INTEGER NOT NULL DEFAULT 0,
    updated_at   INTEGER NOT NULL DEFAULT 0,
    published_at INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_articles_site ON articles (site_id);

-- Key/value app settings (e.g. the Anthropic API key + model).
CREATE TABLE IF NOT EXISTS settings (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT NOT NULL,
    value      TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL DEFAULT 0
);
