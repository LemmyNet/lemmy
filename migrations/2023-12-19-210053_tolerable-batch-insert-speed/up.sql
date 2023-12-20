-- Change triggers to run once per statement instead of once per row

-- post_aggregates_post trigger doesn't need to handle deletion because the post_id column has ON DELETE CASCADE

CREATE OR REPLACE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id)
    SELECT
        id,
        published,
        published,
        published,
        community_id,
        creator_id,
        (SELECT community.instance_id FROM community WHERE community.id = community_id LIMIT 1)
    FROM
        new_post;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION community_aggregates_post_count_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE community_aggregates
    SET posts = posts + post_group.count
    FROM (SELECT community_id, count(*) FROM new_post GROUP BY community_id) post_group
    WHERE community_aggregates.community_id = post_group.community_id;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION person_aggregates_post_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE person_aggregates
    SET post_count = post_count + post_group.count
    FROM (SELECT creator_id, count(*) FROM new_post GROUP BY creator_id) post_group
    WHERE person_aggregates.person_id = post_group.creator_id;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER post_aggregates_post
    AFTER INSERT ON post
    REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE PROCEDURE post_aggregates_post ();

-- Don't run old trigger for insert

CREATE OR REPLACE TRIGGER community_aggregates_post_count
    AFTER DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_post_count ();

CREATE OR REPLACE TRIGGER community_aggregates_post_count_insert
    AFTER INSERT ON post
    REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE PROCEDURE community_aggregates_post_count_insert ();

DROP FUNCTION IF EXISTS site_aggregates_community_delete CASCADE;

CREATE OR REPLACE FUNCTION site_aggregates_post_update ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (was_restored_or_created (TG_OP, OLD, NEW)) THEN
        UPDATE
            site_aggregates sa
        SET
            posts = posts + 1
        FROM
            site s
        WHERE
            sa.site_id = s.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION site_aggregates_post_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        site_aggregates sa
    SET
        posts = posts + (SELECT count(*) FROM new_post)
    FROM
        site s
    WHERE
        sa.site_id = s.id;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER site_aggregates_post_update
    AFTER UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_post_update ();

CREATE OR REPLACE TRIGGER site_aggregates_post_insert
    AFTER INSERT ON post
    REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE PROCEDURE site_aggregates_post_insert ();

CREATE OR REPLACE TRIGGER person_aggregates_post_count
    AFTER DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_post_count ();

CREATE OR REPLACE TRIGGER person_aggregates_post_insert
    AFTER INSERT ON post
    REFERENCING NEW TABLE AS new_post
    FOR EACH STATEMENT
    EXECUTE PROCEDURE person_aggregates_post_insert ();

-- Avoid running hash function and random number generation for default ap_id

CREATE SEQUENCE IF NOT EXISTS changeme_seq AS bigint CYCLE;

CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/seq/' || nextval('changeme_seq')::text;
$$;

