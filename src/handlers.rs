//! Commands exposed to the frontend (typed via `rata codegen`).

use elyra::{command, Ctx, Database, EventBus};
use serde::{Deserialize, Serialize};

use crate::models::{self, Article, Site};
use crate::schedule;

// --- Sites ------------------------------------------------------------------

#[command]
pub async fn list_sites(ctx: Ctx) -> Result<Vec<Site>, String> {
    Site::query()
        .order_by("name")
        .get(&ctx.get::<Database>())
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn get_site(ctx: Ctx, id: i64) -> Result<Option<Site>, String> {
    Site::find(&ctx.get::<Database>(), id)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub async fn save_site(ctx: Ctx, mut site: Site) -> Result<Site, String> {
    let db = ctx.get::<Database>();
    if site.id == 0 {
        // Due right away so the first article generates on the next tick.
        if site.next_run_at == 0 {
            site.next_run_at = schedule::now();
        }
        site.insert(&db).await.map_err(|e| e.to_string())?;
    } else {
        site.update(&db).await.map_err(|e| e.to_string())?;
    }
    ctx.get::<EventBus>().emit("sites", &site.id).ok();
    Ok(site)
}

#[command]
pub async fn delete_site(ctx: Ctx, id: i64) -> Result<(), String> {
    let db = ctx.get::<Database>();
    // Remove the site's articles first.
    let articles = Article::query()
        .where_eq("site_id", id)
        .get(&db)
        .await
        .map_err(|e| e.to_string())?;
    for article in &articles {
        article.delete(&db).await.map_err(|e| e.to_string())?;
    }
    if let Some(site) = Site::find(&db, id).await.map_err(|e| e.to_string())? {
        site.delete(&db).await.map_err(|e| e.to_string())?;
    }
    ctx.get::<EventBus>().emit("sites", &id).ok();
    Ok(())
}

// --- Articles ---------------------------------------------------------------

#[command]
pub async fn list_articles(ctx: Ctx, site_id: i64) -> Result<Vec<Article>, String> {
    Article::query()
        .where_eq("site_id", site_id)
        .order_by_desc("id")
        .get(&ctx.get::<Database>())
        .await
        .map_err(|e| e.to_string())
}

/// Generate one article now (and auto-publish if the site is configured to).
#[command]
pub async fn generate_now(ctx: Ctx, site_id: i64) -> Result<Article, String> {
    let db = ctx.get::<Database>();
    let bus = ctx.get::<EventBus>();
    let site = Site::find(&db, site_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("site not found")?;
    schedule::generate_and_store(&db, &bus, &site).await
}

/// (Re)publish an existing article to its site.
#[command]
pub async fn publish_article(ctx: Ctx, article_id: i64) -> Result<Article, String> {
    let db = ctx.get::<Database>();
    let bus = ctx.get::<EventBus>();
    let mut article = Article::find(&db, article_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("article not found")?;
    let site = Site::find(&db, article.site_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("site not found")?;
    schedule::publish_article(&db, &bus, &mut article, &site).await;
    Ok(article)
}

/// Edit an article's title, excerpt, and body (keeps slug/status/image).
#[command]
pub async fn update_article(
    ctx: Ctx,
    article_id: i64,
    title: String,
    excerpt: String,
    body: String,
    theme: String,
    category: String,
) -> Result<Article, String> {
    let db = ctx.get::<Database>();
    let mut article = Article::find(&db, article_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("article not found")?;
    article.title = title;
    article.excerpt = excerpt;
    article.body = body;
    article.theme = theme;
    article.category = category;
    article.update(&db).await.map_err(|e| e.to_string())?;
    ctx.get::<EventBus>().emit("articles", &article.site_id).ok();
    Ok(article)
}

/// (Re)generate just the cover image for an existing article.
#[command]
pub async fn regenerate_image(ctx: Ctx, article_id: i64) -> Result<Article, String> {
    let db = ctx.get::<Database>();
    let mut article = Article::find(&db, article_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("article not found")?;
    let site = Site::find(&db, article.site_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("site not found")?;
    let data_url = crate::image::generate(&db, &site, &article.title, &article.theme).await?;
    article.image = data_url;
    article.update(&db).await.map_err(|e| e.to_string())?;
    ctx.get::<EventBus>().emit("articles", &article.site_id).ok();
    Ok(article)
}

#[command]
pub async fn delete_article(ctx: Ctx, article_id: i64) -> Result<(), String> {
    let db = ctx.get::<Database>();
    if let Some(article) = Article::find(&db, article_id).await.map_err(|e| e.to_string())? {
        let site_id = article.site_id;
        article.delete(&db).await.map_err(|e| e.to_string())?;
        ctx.get::<EventBus>().emit("articles", &site_id).ok();
    }
    Ok(())
}

// --- Settings ---------------------------------------------------------------

#[derive(Serialize, Deserialize, specta::Type)]
pub struct SettingsView {
    pub anthropic_key_set: bool,
    pub model: String,
    pub openai_key_set: bool,
    pub image_model: String,
}

#[command]
pub async fn get_settings(ctx: Ctx) -> SettingsView {
    let db = ctx.get::<Database>();
    let key = models::get_setting(&db, "anthropic_api_key").await.unwrap_or_default();
    let model = models::get_setting(&db, "anthropic_model").await.unwrap_or_default();
    let openai = models::get_setting(&db, "openai_api_key").await.unwrap_or_default();
    let image_model = models::get_setting(&db, "image_model").await.unwrap_or_default();
    SettingsView {
        anthropic_key_set: !key.trim().is_empty() || std::env::var("ANTHROPIC_API_KEY").is_ok(),
        model: if model.trim().is_empty() {
            "claude-sonnet-5".into()
        } else {
            model
        },
        openai_key_set: !openai.trim().is_empty() || std::env::var("OPENAI_API_KEY").is_ok(),
        image_model: if image_model.trim().is_empty() {
            "gpt-image-1".into()
        } else {
            image_model
        },
    }
}

/// Save settings. Empty key fields leave the stored key unchanged.
#[command]
pub async fn save_settings(
    ctx: Ctx,
    anthropic_key: String,
    model: String,
    openai_key: String,
    image_model: String,
) -> Result<(), String> {
    let db = ctx.get::<Database>();
    if !anthropic_key.trim().is_empty() {
        models::set_setting(&db, "anthropic_api_key", anthropic_key.trim()).await?;
    }
    if !model.trim().is_empty() {
        models::set_setting(&db, "anthropic_model", model.trim()).await?;
    }
    if !openai_key.trim().is_empty() {
        models::set_setting(&db, "openai_api_key", openai_key.trim()).await?;
    }
    if !image_model.trim().is_empty() {
        models::set_setting(&db, "image_model", image_model.trim()).await?;
    }
    Ok(())
}
