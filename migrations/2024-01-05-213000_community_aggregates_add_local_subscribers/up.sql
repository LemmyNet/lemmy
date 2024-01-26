-- Couldn't find a way to put subscribers_local right after subscribers except recreating the table.
ALTER TABLE community_aggregates
    ADD COLUMN subscribers_local bigint NOT NULL DEFAULT 0;

-- update initial value
-- update by counting local persons who follow communities.
WITH follower_counts AS (
    SELECT
        community_id,
        count(*) AS local_sub_count
    FROM
        community_follower cf
        JOIN person p ON p.id = cf.person_id
    WHERE
        p.local = TRUE
    GROUP BY
        community_id)
UPDATE
    community_aggregates ca
SET
    subscribers_local = local_sub_count
FROM
    follower_counts
WHERE
    ca.community_id = follower_counts.community_id;

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
            AND person.local IS NOT NULL;
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
            AND person.local IS NOT NULL;
    END IF;
    RETURN NULL;
END
$$;

-- to be able to join person on the trigger above, we need to run it before the person is deleted: https://github.com/LemmyNet/lemmy/pull/4166#issuecomment-1874095856
CREATE FUNCTION delete_follow_before_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM community_follower AS c
    WHERE c.person_id = OLD.id;
    RETURN OLD;
END;
$$;

CREATE TRIGGER delete_follow_before_person
    BEFORE DELETE ON person
    FOR EACH ROW
    EXECUTE FUNCTION delete_follow_before_person ();

