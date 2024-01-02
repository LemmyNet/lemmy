-- Couldn't find a way to put subscribers_local right after subscribers except recreating the table.
ALTER TABLE community_aggregates
    ADD COLUMN subscribers_local bigint NOT NULL DEFAULT 0;

-- update initial value
UPDATE
    community_aggregates ca
SET
    subscribers_local = (
        SELECT
            COUNT(*)
        FROM
            community_follower cf
        WHERE
            cf.community_id = ca.community_id
            AND (
                SELECT
                    local
                FROM
                    person
                WHERE
                    person.id = cf.person_id));

-- subscribers should be updated only when a local community is followed by a local or remote person
-- subscribers_local should be updated only when a local person follows a local or remote community
CREATE OR REPLACE FUNCTION community_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates ca
        SET
            subscribers = subscribers + community.local::int,
            subscribers_local = subscribers_local + person.local::int
        FROM
            community
        LEFT JOIN person ON person.id = NEW.person_id
    WHERE
        community.id = NEW.community_id
            AND community.id = ca.community_id
            AND person IS NOT NULL;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates ca
        SET
            subscribers = subscribers - community.local::int,
            subscribers_local = subscribers_local - person.local::int
        FROM
            community
        LEFT JOIN person ON person.id = OLD.person_id
    WHERE
        community.id = OLD.community_id
            AND community.id = ca.community_id
            AND person IS NOT NULL;
    END IF;
    RETURN NULL;
END
$$;

