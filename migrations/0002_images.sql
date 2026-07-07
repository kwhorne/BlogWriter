-- Per-site image generation settings.
ALTER TABLE sites ADD COLUMN image_enabled INTEGER NOT NULL DEFAULT 1;
ALTER TABLE sites ADD COLUMN image_style TEXT NOT NULL DEFAULT 'flat isometric';
ALTER TABLE sites ADD COLUMN image_format TEXT NOT NULL DEFAULT '16:9';

-- Generated cover image, stored as a web-optimized data URL (data:image/jpeg;base64,...).
ALTER TABLE articles ADD COLUMN image TEXT NOT NULL DEFAULT '';
