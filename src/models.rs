//! Database models: registered Laravel `Site`s, generated `Article`s, and
//! key/value `Setting`s.

use elyra::{Database, Model};
use serde::{Deserialize, Serialize};

/// A registered Laravel site that receives generated articles.
#[derive(Model, Serialize, Deserialize, specta::Type, Clone, Debug)]
#[model(table = "sites", timestamps, has_many(Article, fk = "site_id", as = "articles"))]
pub struct Site {
    pub id: i64,
    pub name: String,
    /// What the site is about (audience, purpose, voice) — steers generation.
    pub description: String,
    /// Base URL of the Laravel site, e.g. `https://blog.example.com`.
    pub base_url: String,
    /// API path that accepts new articles (default `/api/articles`).
    pub api_path: String,
    /// Bearer token for the Laravel endpoint.
    pub token: String,
    /// Comma-separated topics/themes to write about.
    pub themes: String,
    pub tone: String,
    pub language: String,
    /// `hourly` | `daily` | `weekly` | `monthly` | `manual`.
    pub cadence: String,
    pub active: bool,
    /// Publish immediately after generating (vs. leave as a draft).
    pub auto_publish: bool,
    /// Generate a cover image for each article.
    pub image_enabled: bool,
    /// Illustration style, e.g. `flat isometric` or `risograph grain`.
    pub image_style: String,
    /// Aspect ratio, e.g. `16:9`.
    pub image_format: String,
    /// Unix seconds: when the next article is due.
    pub next_run_at: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A generated article and its publish status.
#[derive(Model, Serialize, Deserialize, specta::Type, Clone, Debug)]
#[model(table = "articles", timestamps, belongs_to(Site, fk = "site_id"))]
pub struct Article {
    pub id: i64,
    pub site_id: i64,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub body: String,
    /// The topic the article was generated for (drives rotation/generation).
    pub theme: String,
    /// Blog category to file the article under (defaults to the theme).
    pub category: String,
    /// `draft` | `published` | `failed`.
    pub status: String,
    /// Id returned by the Laravel site on publish.
    pub remote_id: String,
    pub error: String,
    /// Web-optimized cover image as a data URL (`data:image/jpeg;base64,...`).
    pub image: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub published_at: i64,
}

/// A key/value application setting.
#[derive(Model, Serialize, Deserialize, specta::Type, Clone, Debug)]
#[model(table = "settings", timestamps)]
pub struct Setting {
    pub id: i64,
    pub name: String,
    pub value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Read a setting value by name.
pub async fn get_setting(db: &Database, name: &str) -> Option<String> {
    Setting::query()
        .where_eq("name", name)
        .first(db)
        .await
        .ok()
        .flatten()
        .map(|s| s.value)
}

/// Upsert a setting value by name.
pub async fn set_setting(db: &Database, name: &str, value: &str) -> Result<(), String> {
    match Setting::query()
        .where_eq("name", name)
        .first(db)
        .await
        .map_err(|e| e.to_string())?
    {
        Some(mut existing) => {
            existing.value = value.to_string();
            existing.update(db).await.map_err(|e| e.to_string())
        }
        None => {
            let mut created = Setting {
                id: 0,
                name: name.to_string(),
                value: value.to_string(),
                created_at: 0,
                updated_at: 0,
            };
            created.insert(db).await.map_err(|e| e.to_string())
        }
    }
}
