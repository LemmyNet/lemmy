ALTER TABLE community_aggregates
    DROP COLUMN subscribers_local;

-- old function from migrations/2023-10-02-145002_community_followers_count_federated/up.sql
-- The subscriber count should only be updated for local communities. For remote
-- communities it is read over federation from the origin instance.
CREATE OR REPLACE FUNCTION community_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates
        SET
            subscribers = subscribers + 1
        FROM
            community
        WHERE
            community.id = community_id
            AND community.local
            AND community_id = NEW.community_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates
        SET
            subscribers = subscribers - 1
        FROM
            community
        WHERE
            community.id = community_id
            AND community.local
            AND community_id = OLD.community_id;
    END IF;
    RETURN NULL;
END
$$;

