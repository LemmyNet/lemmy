ALTER TABLE multi_community
    ADD COLUMN subscribers int NOT NULL DEFAULT 0,
    ADD COLUMN subscribers_local int NOT NULL DEFAULT 0,
    ADD COLUMN communities int NOT NULL DEFAULT 0;

-- Add indexes for all the sorts, to somewhat match the ones on community
CREATE INDEX idx_multi_community_lower_name ON multi_community (lower(name::text) DESC, id DESC);

CREATE INDEX idx_multi_community_subscribers ON multi_community (subscribers DESC, id DESC);

CREATE INDEX idx_multi_community_subscribers_local ON multi_community (subscribers_local DESC, id DESC);

CREATE INDEX idx_multi_community_communities ON multi_community (communities DESC, id DESC);

CREATE INDEX idx_multi_community_published ON multi_community (published_at DESC, id DESC);

