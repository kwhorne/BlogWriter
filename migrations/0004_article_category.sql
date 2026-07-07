-- Blog category (where the article is filed); distinct from `theme` (the topic).
ALTER TABLE articles ADD COLUMN category TEXT NOT NULL DEFAULT '';
