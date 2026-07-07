# Image generation

BlogWriter can generate an editorial cover image for every article. Claude is
text-only, so images use a **separate provider**: the OpenAI Images API.

## Requirements

- An OpenAI API key (`Settings → API keys`, or `$OPENAI_API_KEY`)
- **Image enabled** toggled on for the site (default: on)

If no key is set or generation fails, the article is still created — just
without a cover. Image errors never block the text pipeline.

## Models

| Model | Response type | Landscape | Portrait | Square |
|---|---|---|---|---|
| `gpt-image-1` (default) | inline base64 | 1536×1024 | 1024×1536 | 1024×1024 |
| `dall-e-*` | hosted URL (downloaded) | 1792×1024 | 1024×1792 | 1024×1024 |

The generation size is chosen automatically from the site's aspect ratio:
ratios above ~1.2 use the landscape size, below ~0.83 portrait, otherwise
square. The exact final ratio is achieved by cropping (see below).

## Per-site image options

| Option | Default | Effect |
|---|---|---|
| **Image style** | `flat isometric` | Free-text style injected into the prompt (e.g. `risograph grain`, `watercolor`, `line art`) |
| **Image format** | `16:9` | Target aspect ratio as `W:H`; invalid values fall back to 16:9 |

## The prompt

Each image is prompted as an *editorial cover illustration* for the article's
title and theme, in the site's style, with a consistent color palette (the
"Grove" palette: warm beige, terracotta, dove blue, matte moss green, brass,
ink black), a clean composition, and **no text, words, or logos**.

This gives all covers across a site a coherent visual identity while the
subject varies per article.

## Optimization pipeline

Raw generations are large PNGs at fixed provider sizes. Before storage,
BlogWriter:

1. **Center-crops** to the site's exact aspect ratio
2. **Downscales** to at most 1600 px wide (Lanczos3)
3. **Re-encodes** as JPEG at quality 82

The result is stored on the article as a data URL
(`data:image/jpeg;base64,...`) and sent as the `image` field when publishing
— see [laravel-endpoint.md](laravel-endpoint.md) for how to decode and store
it on the Laravel side.

## Regenerating a cover

Don't like the image? Use **Regenerate image** on an article to produce a new
cover without touching the text. Re-publish afterwards to update the site.
