DROP INDEX idx_post_trigram;

CREATE INDEX IF NOT EXISTS idx_post_trigram ON post USING gin (name gin_trgm_ops, body gin_trgm_ops);

