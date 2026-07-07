//! Cover-image generation via OpenAI Images, cropped to the site's aspect ratio
//! and re-encoded as a web-optimized JPEG data URL.
//!
//! Anthropic/Claude is text-only, so images use a separate provider (OpenAI
//! `gpt-image-1`) with its own key (`openai_api_key` setting or `$OPENAI_API_KEY`).

use base64::Engine;
use image::{GenericImageView, ImageEncoder};
use serde_json::json;

use elyra::Database;

use crate::models::{self, Site};

const OPENAI_URL: &str = "https://api.openai.com/v1/images/generations";
const DEFAULT_MODEL: &str = "gpt-image-1";

/// The Grove series palette, applied to every generated image.
const PALETTE: &str = "Use the Grove palette only: warm beige, terracotta, \
    dove blue, matte moss green, brass, and ink black.";

pub async fn api_key(db: &Database) -> Result<String, String> {
    if let Some(key) = models::get_setting(db, "openai_api_key").await {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    std::env::var("OPENAI_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| "No OpenAI API key set (Settings → image key, or $OPENAI_API_KEY)".into())
}

/// Aspect ratio (w/h) parsed from a `"16:9"`-style string; defaults to 16:9.
fn aspect_ratio(format: &str) -> f32 {
    format
        .split_once(':')
        .and_then(|(a, b)| Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?)))
        .filter(|(_, h)| *h > 0.0)
        .map(|(w, h)| w / h)
        .unwrap_or(16.0 / 9.0)
}

/// Closest supported generation size for a model + aspect ratio.
/// (`dall-e-3` and `gpt-image-1` accept different landscape/portrait sizes.)
fn size_for(model: &str, ratio: f32) -> &'static str {
    let dalle = model.contains("dall-e");
    if ratio > 1.2 {
        if dalle {
            "1792x1024"
        } else {
            "1536x1024"
        }
    } else if ratio < 0.83 {
        if dalle {
            "1024x1792"
        } else {
            "1024x1536"
        }
    } else {
        "1024x1024"
    }
}

/// Generate a cover image and return it as a `data:image/jpeg;base64,...` URL.
pub async fn generate(db: &Database, site: &Site, title: &str, theme: &str) -> Result<String, String> {
    let key = api_key(db).await?;
    let model = models::get_setting(db, "image_model")
        .await
        .filter(|m| !m.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MODEL.to_string());
    let ratio = aspect_ratio(&site.image_format);

    let prompt = format!(
        "Editorial cover illustration for a blog article titled \"{title}\", about {theme}. \
         Style: {style}. {palette} Clean, modern composition with generous negative space, \
         no text, no words, no logos. Designed as a {format} banner.",
        style = site.image_style,
        palette = PALETTE,
        format = site.image_format,
    );

    let payload = json!({ "model": model, "prompt": prompt, "n": 1, "size": size_for(&model, ratio) });

    let client = reqwest::Client::new();
    let resp = client
        .post(OPENAI_URL)
        .bearer_auth(&key)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = resp.status();
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        return Err(format!("OpenAI images {status}: {text}"));
    }

    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("bad API response: {e}"))?;
    let item = &value["data"][0];

    // Models return either inline base64 (gpt-image-1) or a URL (dall-e-*).
    let raw = if let Some(b64) = item["b64_json"].as_str() {
        base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|e| format!("base64 decode: {e}"))?
    } else if let Some(url) = item["url"].as_str() {
        client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("image download failed: {e}"))?
            .bytes()
            .await
            .map_err(|e| e.to_string())?
            .to_vec()
    } else {
        let snippet: String = text.chars().take(400).collect();
        return Err(format!("no image in response: {snippet}"));
    };

    let jpeg = optimize(&raw, ratio)?;
    let encoded = base64::engine::general_purpose::STANDARD.encode(&jpeg);
    Ok(format!("data:image/jpeg;base64,{encoded}"))
}

/// Center-crop `bytes` to `ratio`, downscale for web (max 1600px wide), and
/// re-encode as JPEG.
pub fn optimize(bytes: &[u8], ratio: f32) -> Result<Vec<u8>, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("decode image: {e}"))?;
    let (w, h) = img.dimensions();
    let current = w as f32 / h as f32;

    let (crop_w, crop_h) = if current > ratio {
        (((h as f32) * ratio).round() as u32, h)
    } else {
        (w, ((w as f32) / ratio).round() as u32)
    };
    let x = (w.saturating_sub(crop_w)) / 2;
    let y = (h.saturating_sub(crop_h)) / 2;
    let cropped = img.crop_imm(x, y, crop_w.max(1), crop_h.max(1));

    let target_w = crop_w.clamp(1, 1600);
    let target_h = (((target_w as f32) / ratio).round() as u32).max(1);
    let resized = cropped.resize_exact(target_w, target_h, image::imageops::FilterType::Lanczos3);
    let rgb = resized.to_rgb8();

    let mut out = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 82)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("encode jpeg: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn optimize_crops_to_aspect_and_shrinks() {
        // A 1200x1200 source PNG.
        let src = image::RgbImage::from_pixel(1200, 1200, image::Rgb([200, 120, 80]));
        let mut png = Vec::new();
        image::DynamicImage::ImageRgb8(src)
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();

        let jpeg = optimize(&png, 16.0 / 9.0).unwrap();
        let decoded = image::load_from_memory(&jpeg).unwrap();
        let (w, h) = decoded.dimensions();
        let ratio = w as f32 / h as f32;
        assert!((ratio - 16.0 / 9.0).abs() < 0.05, "aspect {ratio}");
        assert!(w <= 1600);
        assert!(!jpeg.is_empty());
    }

    #[test]
    fn aspect_parsing() {
        assert!((aspect_ratio("16:9") - 1.777).abs() < 0.01);
        assert!((aspect_ratio("1:1") - 1.0).abs() < 0.01);
        assert!((aspect_ratio("bogus") - 1.777).abs() < 0.01);
    }
}
