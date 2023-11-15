-- Couldn't find a way to put local_subscribers right after subscribers
ALTER TABLE community_aggregates
    ADD COLUMN local_subscribers int8 NOT NULL DEFAULT 0;

-- update initial value
UPDATE
    community_aggregates
SET
    local_subscribers = (
        SELECT
            COUNT(*)
        FROM
            community_follower
        WHERE
            community_follower.community_id = community_aggregates.community_id
            AND community_follower.person_id IN (
                SELECT
                    id
                FROM
                    person
                WHERE
                    local
            )
    )
;


CREATE OR REPLACE FUNCTION community_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates
        SET
            subscribers = subscribers + 1,
            local_subscribers = local_subscribers + 1
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
            subscribers = subscribers - 1,
            local_subscribers = local_subscribers - 1
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

