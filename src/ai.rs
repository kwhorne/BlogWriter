//! Article generation via the Anthropic Messages API.

use elyra::Database;
use serde::Deserialize;
use serde_json::json;

use crate::models::{self, Site};

/// Default model — Claude Sonnet 5. Override in Settings if you need a snapshot
/// pin (e.g. `claude-sonnet-5-YYYYMMDD`).
const DEFAULT_MODEL: &str = "claude-sonnet-5";
const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

/// A generated article, parsed from the model's JSON response.
#[derive(Debug, Deserialize)]
pub struct Generated {
    pub title: String,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub body: String,
}

/// Resolve the Anthropic API key: the `anthropic_api_key` setting, else the
/// `ANTHROPIC_API_KEY` env var.
pub async fn api_key(db: &Database) -> Result<String, String> {
    if let Some(key) = models::get_setting(db, "anthropic_api_key").await {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| "No Anthropic API key set (Settings → API key, or $ANTHROPIC_API_KEY)".into())
}

/// Generate one article for `site` on the given `theme`.
pub async fn generate(db: &Database, site: &Site, theme: &str) -> Result<Generated, String> {
    let key = api_key(db).await?;
    let model = models::get_setting(db, "anthropic_model")
        .await
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());

    let context = if site.description.trim().is_empty() {
        String::new()
    } else {
        format!(
            "About the site (audience, purpose, and what it wants to convey): {}\n\
             Write so the article fits this site's focus.\n",
            site.description.trim()
        )
    };

    let prompt = format!(
        "Write an original, engaging blog article in {language}.\n\
         {context}\
         Theme / topic: \"{theme}\".\n\
         Tone: {tone}.\n\
         Return ONLY minified JSON (no markdown fences, no commentary) with exactly these keys:\n\
         \"title\" (string), \"excerpt\" (1-2 sentence summary), \"body\" (the full article in Markdown, ~600-900 words).",
        language = site.language,
        context = context,
        tone = site.tone,
    );

    let payload = json!({
        "model": model,
        "max_tokens": 4096,
        "messages": [{ "role": "user", "content": prompt }],
    });

    let resp = reqwest::Client::new()
        .post(ANTHROPIC_URL)
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("Anthropic API {status}: {text}"));
    }

    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("bad API response: {e}"))?;

    if let Some(err) = value.get("error") {
        return Err(format!("Anthropic error: {err}"));
    }

    // Concatenate every text block, skipping non-text blocks (e.g. "thinking").
    let combined = value["content"]
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
                .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();

    if combined.trim().is_empty() {
        let snippet: String = text.chars().take(500).collect();
        return Err(format!("no text content in Anthropic response: {snippet}"));
    }

    parse_generated(&combined)
}

/// Parse the model's reply into a [`Generated`], tolerating stray prose or code
/// fences by extracting the outermost JSON object.
fn parse_generated(raw: &str) -> Result<Generated, String> {
    let slice = match (raw.find('{'), raw.rfind('}')) {
        (Some(start), Some(end)) if end > start => &raw[start..=end],
        _ => raw,
    };
    serde_json::from_str::<Generated>(slice).map_err(|e| format!("model did not return valid JSON: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fenced_json() {
        let raw = "```json\n{\"title\":\"Hi\",\"excerpt\":\"e\",\"body\":\"# b\"}\n```";
        let g = parse_generated(raw).unwrap();
        assert_eq!(g.title, "Hi");
        assert_eq!(g.body, "# b");
    }

    #[test]
    fn parses_json_with_prose() {
        let raw = "Sure! Here you go: {\"title\":\"T\",\"excerpt\":\"\",\"body\":\"B\"} hope that helps";
        let g = parse_generated(raw).unwrap();
        assert_eq!(g.title, "T");
    }
}
