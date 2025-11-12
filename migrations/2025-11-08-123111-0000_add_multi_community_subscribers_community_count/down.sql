DROP INDEX idx_multi_community_lower_name;

DROP INDEX idx_multi_community_subscribers;

DROP INDEX idx_multi_community_subscribers_local;

DROP INDEX idx_multi_community_communities;

DROP INDEX idx_multi_community_published;

ALTER TABLE multi_community
    DROP COLUMN subscribers,
    DROP COLUMN subscribers_local,
    DROP COLUMN communities;

