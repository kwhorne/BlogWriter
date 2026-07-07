-- What the site is about (audience, purpose, voice) — steers article generation.
ALTER TABLE sites ADD COLUMN description TEXT NOT NULL DEFAULT '';
