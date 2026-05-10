DROP INDEX idx_person_trigram;

CREATE INDEX idx_person_trigram ON person USING gin (name gin_trgm_ops, display_name gin_trgm_ops);

DROP INDEX idx_community_trigram;

CREATE INDEX idx_community_trigram ON community USING gin (name gin_trgm_ops, title gin_trgm_ops);

DROP INDEX idx_person_post_score, idx_person_comment_score, idx_multi_community_trigram;

