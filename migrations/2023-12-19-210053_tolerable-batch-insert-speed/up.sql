-- Change post_aggregates trigger to run once per statement instead of once per row.
-- The trigger doesn't need to handle deletion because the post_id column has ON DELETE CASCADE.

CREATE OR REPLACE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id)
    SELECT
        new_post.id,
        new_post.published,
        new_post.published,
        new_post.published,
        new_post.community_id,
        new_post.creator_id,
        (SELECT community.instance_id FROM community WHERE community.id = new_post.community_id LIMIT 1)
    FROM
        new_post;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER post_aggregates_post
    AFTER INSERT ON post
    REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE PROCEDURE post_aggregates_post ();

-- Avoid running hash function and random number generation for default ap_id

CREATE SEQUENCE IF NOT EXISTS changeme_seq AS bigint CYCLE;

CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/seq/' || nextval('changeme_seq')::text;
$$;

