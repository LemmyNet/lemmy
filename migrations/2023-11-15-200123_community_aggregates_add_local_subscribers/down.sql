ALTER TABLE community_aggregates
		DROP COLUMN local_subscribers;

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

