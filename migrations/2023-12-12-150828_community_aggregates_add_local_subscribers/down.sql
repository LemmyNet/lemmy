ALTER TABLE community_aggregates
    DROP COLUMN local_subscribers;

DROP TRIGGER IF EXISTS community_aggregates_local_subscriber_count ON community_follower;

DROP FUNCTION IF EXISTS community_aggregates_local_subscriber_count;

