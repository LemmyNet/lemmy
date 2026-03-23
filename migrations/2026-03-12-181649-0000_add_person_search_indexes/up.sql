-- Adds some missing indexes to the person and multi_community tables, used for searching
CREATE INDEX idx_person_post_score ON person (post_score DESC);

CREATE INDEX idx_person_comment_score ON person (comment_score DESC);

DROP INDEX idx_person_trigram;

CREATE INDEX idx_person_trigram ON person USING gin (name gin_trgm_ops, display_name gin_trgm_ops, bio gin_trgm_ops);

CREATE INDEX idx_multi_community_trigram ON multi_community USING gin (name gin_trgm_ops, title gin_trgm_ops, summary gin_trgm_ops, sidebar gin_trgm_ops);

-- The community one used some wrong terms.
DROP INDEX idx_community_trigram;

CREATE INDEX idx_community_trigram ON community USING gin (name gin_trgm_ops, title gin_trgm_ops, summary gin_trgm_ops, sidebar gin_trgm_ops);

