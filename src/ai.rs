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
         \"title\" (string), \"excerpt\" (1-2 sentence summary), \"body\" (the full article in Markdown, ~600-900 words).\n\
         Inside JSON strings use only valid JSON escapes; never backslash-escape Markdown characters like * _ ( ) [ ] or #.",
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
/// fences by extracting the outermost JSON object. If parsing fails on an
/// invalid escape (models sometimes Markdown-escape characters like `\*`
/// inside JSON strings), retry with those escapes repaired.
fn parse_generated(raw: &str) -> Result<Generated, String> {
    let slice = match (raw.find('{'), raw.rfind('}')) {
        (Some(start), Some(end)) if end > start => &raw[start..=end],
        _ => raw,
    };
    match serde_json::from_str::<Generated>(slice) {
        Ok(g) => Ok(g),
        Err(first_err) => serde_json::from_str::<Generated>(&repair_invalid_escapes(slice))
            .map_err(|_| format!("model did not return valid JSON: {first_err}")),
    }
}

/// Repair JSON-invalid escape sequences by turning the backslash into a
/// literal one (`\*` → `\\*`). Valid escapes (`\"`, `\\`, `\/`, `\b`, `\f`,
/// `\n`, `\r`, `\t`, and `\uXXXX` with four hex digits) pass through intact.
fn repair_invalid_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.peek().copied() {
            Some(n) if matches!(n, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') => {
                out.push('\\');
                out.push(n);
                chars.next();
            }
            Some('u') => {
                let hex: String = chars.clone().skip(1).take(4).collect();
                if hex.len() == 4 && hex.chars().all(|h| h.is_ascii_hexdigit()) {
                    out.push('\\'); // valid \uXXXX — leave as-is
                } else {
                    out.push_str("\\\\"); // malformed unicode escape
                }
            }
            _ => out.push_str("\\\\"), // lone/invalid escape → literal backslash
        }
    }
    out
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

    #[test]
    fn repairs_markdown_escapes() {
        // `\*` and `\-` are valid Markdown escaping but invalid JSON escapes.
        let raw = r#"{"title":"T","excerpt":"e","body":"a \* b \- c"}"#;
        let g = parse_generated(raw).unwrap();
        assert_eq!(g.body, r"a \* b \- c");
    }

    #[test]
    fn keeps_valid_escapes_intact() {
        let raw = r#"{"title":"T \u00e9","excerpt":"","body":"line\nnext \\ \"q\""}"#;
        let g = parse_generated(raw).unwrap();
        assert_eq!(g.title, "T \u{e9}");
        assert_eq!(g.body, "line\nnext \\ \"q\"");
    }

    #[test]
    fn repairs_malformed_unicode_escape() {
        let raw = r#"{"title":"T","excerpt":"","body":"bad \uZZ99 escape"}"#;
        let g = parse_generated(raw).unwrap();
        assert_eq!(g.body, r"bad \uZZ99 escape");
    }
}
