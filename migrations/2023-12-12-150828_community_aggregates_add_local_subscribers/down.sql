ALTER TABLE community_aggregates
    DROP COLUMN subscribers_local;

DROP TRIGGER IF EXISTS community_aggregates_subscriber_local_count ON community_follower;

DROP FUNCTION IF EXISTS community_aggregates_subscriber_local_count;

