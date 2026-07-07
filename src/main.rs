//! BlogWriter — generate blog articles with Anthropic and auto-publish them to
//! registered Laravel sites on a schedule.

mod ai;
mod handlers;
mod image;
mod models;
mod publish;
mod schedule;
mod updater;

#[cfg(test)]
mod db_tests;

use elyra::{commands, App, Database, TrayConfig};

/// The built Svelte frontend, embedded from memory.
#[derive(rust_embed::RustEmbed)]
#[folder = "app/dist"]
struct Assets;

fn db_url() -> String {
    format!(
        "sqlite://{}/blogwriter.db?mode=rwc",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn main() -> elyra::Result<()> {
    let url = db_url();

    // Auto-migrate on startup (skipped in codegen mode, which never opens a window).
    if std::env::var_os("ELYRA_CODEGEN_OUT").is_none() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let db = Database::connect(&url).await.expect("connect db");
            db.migrator(concat!(env!("CARGO_MANIFEST_DIR"), "/migrations"))
                .run()
                .await
                .expect("run migrations");
        });
    }

    App::new()
        .title("BlogWriter")
        .size(1040.0, 720.0)
        .min_size(760.0, 540.0)
        .database(url)
        .provider(schedule::Scheduler)
        .tray(
            TrayConfig::new()
                .tooltip("BlogWriter")
                .item("open", "Open BlogWriter")
                .separator()
                .quit("Quit"),
        )
        .commands(commands![
            handlers::list_sites,
            handlers::get_site,
            handlers::save_site,
            handlers::delete_site,
            handlers::list_articles,
            handlers::generate_now,
            handlers::publish_article,
            handlers::update_article,
            handlers::regenerate_image,
            handlers::delete_article,
            handlers::get_settings,
            handlers::save_settings,
            updater::check_for_update,
            updater::install_update,
        ])
        .assets(elyra::asset_resolver::<Assets>())
        .run()
}
