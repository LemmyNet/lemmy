-- Couldn't find a way to put local_subscribers right after subscribers except recreating the table.
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
            AND (
                SELECT
                    local
                FROM
                    person
                WHERE
                    person.id = community_follower.person_id));

CREATE OR REPLACE FUNCTION community_aggregates_local_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates
        SET
            local_subscribers = local_subscribers + 1
        FROM
            community c
        WHERE
            c.id = NEW.community_id
            AND (
                SELECT
                    local
                FROM
                    person
                WHERE
                    id = NEW.person_id);
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates
        SET
            local_subscribers = local_subscribers - 1
        FROM
            community c
        WHERE
            c.id = OLD.community_id
            AND (
                SELECT
                    local
                FROM
                    person
                WHERE
                    id = OLD.person_id);
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER community_aggregates_local_subscriber_count
    AFTER INSERT OR DELETE ON community_follower
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_local_subscriber_count ();

