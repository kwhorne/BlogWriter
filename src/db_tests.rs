//! End-to-end DB test against BlogWriter's real migration + models (SQLite).

use elyra::Database;

use crate::models::{Article, Site};

async fn setup() -> (std::path::PathBuf, Database) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("blogwriter-test-{nanos}.db"));
    let _ = std::fs::remove_file(&path);
    let db = Database::connect(&format!("sqlite://{}?mode=rwc", path.display()))
        .await
        .unwrap();
    db.migrator(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"))
        .run()
        .await
        .unwrap();
    (path, db)
}

#[tokio::test]
async fn site_and_article_lifecycle() {
    let (path, db) = setup().await;

    let mut site = Site {
        id: 0,
        name: "Blog".into(),
        description: "A friendly blog about subscriptions.".into(),
        base_url: "https://blog.example.com".into(),
        api_path: "/api/articles".into(),
        token: "secret".into(),
        themes: "a, b".into(),
        tone: "friendly".into(),
        language: "English".into(),
        cadence: "daily".into(),
        active: true,
        auto_publish: true,
        image_enabled: true,
        image_style: "flat isometric".into(),
        image_format: "16:9".into(),
        next_run_at: 0,
        created_at: 0,
        updated_at: 0,
    };
    site.insert(&db).await.unwrap();
    assert!(site.id > 0);
    assert!(site.created_at > 0); // timestamps auto-set
    assert!(site.active); // bool roundtrip

    let mut article = Article {
        id: 0,
        site_id: site.id,
        title: "Ten Tips".into(),
        slug: "ten-tips".into(),
        excerpt: "e".into(),
        body: "# body".into(),
        theme: "a".into(),
        category: "a".into(),
        status: "draft".into(),
        remote_id: String::new(),
        error: String::new(),
        image: String::new(),
        created_at: 0,
        updated_at: 0,
        published_at: 0,
    };
    article.insert(&db).await.unwrap();

    // has_many relation
    let articles = site.articles(&db).await.unwrap();
    assert_eq!(articles.len(), 1);
    assert_eq!(articles[0].title, "Ten Tips");

    // belongs_to relation
    let owner = article.site(&db).await.unwrap().unwrap();
    assert_eq!(owner.id, site.id);

    // active-site query (bool bound as 0/1)
    let active = Site::query().where_eq("active", true).get(&db).await.unwrap();
    assert_eq!(active.len(), 1);

    // publish flow: mark published
    article.status = "published".into();
    article.remote_id = "42".into();
    article.update(&db).await.unwrap();
    let reloaded = Article::find(&db, article.id).await.unwrap().unwrap();
    assert_eq!(reloaded.status, "published");
    assert_eq!(reloaded.remote_id, "42");

    let _ = std::fs::remove_file(&path);
}
