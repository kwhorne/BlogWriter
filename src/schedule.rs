//! Cadence math, the generate→publish pipeline, and the background scheduler.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use elyra::{Container, Ctx, Database, EventBus, Provider};

use crate::models::{Article, Site};
use crate::{ai, publish};

/// How often a site produces an article.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Manual,
}

impl Cadence {
    pub fn parse(s: &str) -> Cadence {
        match s.trim().to_ascii_lowercase().as_str() {
            "hourly" => Cadence::Hourly,
            "weekly" => Cadence::Weekly,
            "monthly" => Cadence::Monthly,
            "manual" => Cadence::Manual,
            _ => Cadence::Daily,
        }
    }

    /// Interval in seconds, or `None` for `manual` (never auto-runs).
    pub fn seconds(self) -> Option<i64> {
        match self {
            Cadence::Hourly => Some(3_600),
            Cadence::Daily => Some(86_400),
            Cadence::Weekly => Some(604_800),
            Cadence::Monthly => Some(2_592_000),
            Cadence::Manual => None,
        }
    }
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// The next run time for `cadence` measured from `from`, or `None` for manual.
pub fn next_run(cadence: &str, from: i64) -> Option<i64> {
    Cadence::parse(cadence).seconds().map(|s| from + s)
}

/// A URL-friendly slug from a title.
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for ch in title.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.extend(ch.to_lowercase());
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_end_matches('-').chars().take(80).collect()
}

/// Choose the next theme for a site, rotating through its comma-separated list.
async fn pick_theme(db: &Database, site: &Site) -> String {
    let themes: Vec<&str> = site
        .themes
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .collect();
    if themes.is_empty() {
        return "general updates".to_string();
    }
    let count = Article::query()
        .where_eq("site_id", site.id)
        .get(db)
        .await
        .map(|v| v.len())
        .unwrap_or(0);
    themes[count % themes.len()].to_string()
}

/// Generate an article for `site`, store it, and (if `auto_publish`) publish it.
pub async fn generate_and_store(db: &Database, bus: &EventBus, site: &Site) -> Result<Article, String> {
    let theme = pick_theme(db, site).await;
    let draft = ai::generate(db, site, &theme).await?;

    let mut article = Article {
        id: 0,
        site_id: site.id,
        slug: slugify(&draft.title),
        title: draft.title,
        excerpt: draft.excerpt,
        body: draft.body,
        category: theme.clone(), // default the category to the theme; editable later
        theme,
        status: "draft".into(),
        remote_id: String::new(),
        error: String::new(),
        image: String::new(),
        created_at: 0,
        updated_at: 0,
        published_at: 0,
    };
    article.insert(db).await.map_err(|e| e.to_string())?;

    // Generate a cover image (best-effort; a failure doesn't block the article).
    if site.image_enabled {
        match crate::image::generate(db, site, &article.title, &article.theme).await {
            Ok(data_url) => {
                article.image = data_url;
                let _ = article.update(db).await;
            }
            Err(e) => eprintln!("image: site {} article {}: {e}", site.id, article.id),
        }
    }

    if site.auto_publish {
        publish_article(db, bus, &mut article, site).await;
    }

    bus.emit("articles", &site.id).ok();
    Ok(article)
}

/// Publish an existing article to its site, updating status in place.
pub async fn publish_article(db: &Database, bus: &EventBus, article: &mut Article, site: &Site) {
    match publish::publish(site, article).await {
        Ok(remote_id) => {
            article.status = "published".into();
            article.remote_id = remote_id;
            article.error = String::new();
            article.published_at = now();
        }
        Err(e) => {
            article.status = "failed".into();
            article.error = e;
        }
    }
    let _ = article.update(db).await;
    bus.emit("articles", &site.id).ok();
}

/// Run one due site: generate (+ maybe publish), then advance its schedule.
async fn run_site(db: &Database, bus: &EventBus, mut site: Site) {
    if let Err(e) = generate_and_store(db, bus, &site).await {
        eprintln!("scheduler: site {} failed: {e}", site.id);
    }
    site.next_run_at = next_run(&site.cadence, now()).unwrap_or_else(|| now() + 86_400);
    let _ = site.update(db).await;
    bus.emit("sites", &site.id).ok();
}

/// One scheduler pass: run every active, non-manual site that's due.
async fn tick(db: &Database, bus: &EventBus) {
    let now = now();
    let due = Site::query()
        .where_eq("active", true)
        .where_lte("next_run_at", now)
        .get(db)
        .await;
    let Ok(sites) = due else { return };
    for site in sites {
        if Cadence::parse(&site.cadence) == Cadence::Manual {
            continue;
        }
        run_site(db, bus, site).await;
    }
}

/// Provider that spawns the background scheduler loop on boot.
pub struct Scheduler;

impl Provider for Scheduler {
    fn register(&self, _container: &mut Container) {}

    fn boot(&self, ctx: &Ctx) {
        let db = ctx.get::<Database>();
        let bus = ctx.get::<EventBus>();
        // Runs within the app's tokio runtime (App::run enters it before boot).
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                tick(&db, &bus).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cadence_intervals() {
        assert_eq!(Cadence::parse("hourly").seconds(), Some(3_600));
        assert_eq!(Cadence::parse("daily").seconds(), Some(86_400));
        assert_eq!(Cadence::parse("weekly").seconds(), Some(604_800));
        assert_eq!(Cadence::parse("manual").seconds(), None);
        assert_eq!(Cadence::parse("nonsense").seconds(), Some(86_400)); // defaults to daily
    }

    #[test]
    fn next_run_advances() {
        assert_eq!(next_run("daily", 1000), Some(1000 + 86_400));
        assert_eq!(next_run("manual", 1000), None);
    }

    #[test]
    fn slugs() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("  Multiple   spaces &  symbols  "), "multiple-spaces-symbols");
        assert_eq!(slugify("Ünïcode messes"), "n-code-messes"); // ascii-only, best effort
    }
}
