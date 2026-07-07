//! Publishing articles to a Laravel site's API.
//!
//! Posts JSON to `{base_url}{api_path}` with a bearer token. Articles are
//! stored as Markdown locally but published as **HTML** (rendered here), so
//! receiving sites with rich-text editors (e.g. Flux editor) can store and
//! edit the body directly:
//!
//! ```json
//! { "title": "...", "slug": "...", "excerpt": "...", "body": "...(html)",
//!   "body_markdown": "...(source)", "theme": "...", "status": "published" }
//! ```
//!
//! and respond `2xx` with (optionally) `{ "id": <post id> }`. See
//! `docs/laravel-endpoint.md` for a copy-paste route + controller.

use serde_json::json;

use crate::models::{Article, Site};

/// Render Markdown to HTML (tables + strikethrough enabled, GFM-style).
pub fn markdown_to_html(markdown: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH;
    let parser = Parser::new_ext(markdown, opts);
    let mut out = String::with_capacity(markdown.len() * 2);
    html::push_html(&mut out, parser);
    out
}

/// Publish `article` to `site`. Returns the remote post id (may be empty).
pub async fn publish(site: &Site, article: &Article) -> Result<String, String> {
    let base = site.base_url.trim_end_matches('/');
    let path = if site.api_path.starts_with('/') {
        site.api_path.clone()
    } else {
        format!("/{}", site.api_path)
    };
    let url = format!("{base}{path}");

    let payload = json!({
        "title": article.title,
        "slug": article.slug,
        "excerpt": article.excerpt,
        // The body is authored/stored as Markdown but published as HTML so
        // rich-text editors on the receiving site can edit it directly.
        "body": markdown_to_html(&article.body),
        "body_markdown": article.body,
        "theme": article.theme,
        // Category to file under; falls back to the theme if unset.
        "category": if article.category.trim().is_empty() { &article.theme } else { &article.category },
        "image": article.image, // data URL (data:image/jpeg;base64,...) or empty
        "status": "published",
    });

    // Don't let the client auto-follow redirects: reqwest downgrades POST->GET on
    // 301/302 (which yields "405 GET not supported"). We follow them as POST.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| e.to_string())?;

    let token = site.token.trim();
    let mut url = url;
    let mut resp;
    let mut hops = 0;
    loop {
        let mut req = client
            .post(&url)
            .header("accept", "application/json")
            .json(&payload);
        if !token.is_empty() {
            req = req.bearer_auth(token);
        }
        resp = req.send().await.map_err(|e| format!("request failed: {e}"))?;

        if resp.status().is_redirection() {
            hops += 1;
            if hops > 5 {
                return Err("too many redirects".into());
            }
            let location = resp
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|v| v.to_str().ok())
                .ok_or("redirect without Location header")?;
            url = resp
                .url()
                .join(location)
                .map(|u| u.to_string())
                .unwrap_or_else(|_| location.to_string());
            continue;
        }
        break;
    }

    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("publish {status}: {text}"));
    }

    // Best-effort: pull an id out of the JSON response, whether numeric or string.
    let remote_id = serde_json::from_str::<serde_json::Value>(&text)
        .ok()
        .and_then(|v| {
            let node = if v["id"].is_null() { v["data"]["id"].clone() } else { v["id"].clone() };
            node.as_i64()
                .map(|n| n.to_string())
                .or_else(|| node.as_str().map(|s| s.to_string()))
        })
        .unwrap_or_default();

    Ok(remote_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_renders_to_html() {
        let html = markdown_to_html("# Hi\n\nSome **bold** text.\n\n- a\n- b");
        assert!(html.contains("<h1>Hi</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<li>a</li>"));
    }

    #[test]
    fn markdown_tables_enabled() {
        let html = markdown_to_html("| a | b |\n|---|---|\n| 1 | 2 |");
        assert!(html.contains("<table>"));
    }
}
