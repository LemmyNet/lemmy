CREATE OR REPLACE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, community_id, creator_id, instance_id)
        SELECT
            NEW.id,
            NEW.published,
            NEW.published,
            NEW.published,
            NEW.community_id,
            NEW.creator_id,
            community.instance_id
        FROM
            community
        WHERE
            NEW.community_id = community.id;
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates
        WHERE post_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER post_aggregates_post
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE post_aggregates_post ();

CREATE OR REPLACE TRIGGER community_aggregates_post_count
    AFTER INSERT OR DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_post_count ();

DROP FUNCTION IF EXISTS community_aggregates_post_count_insert CASCADE;

DROP FUNCTION IF EXISTS community_aggregates_post_update CASCADE;

DROP FUNCTION IF EXISTS site_aggregates_post_update CASCADE;

DROP FUNCTION IF EXISTS person_aggregates_post_insert CASCADE;

CREATE OR REPLACE FUNCTION site_aggregates_post_insert ()
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

CREATE OR REPLACE TRIGGER site_aggregates_post_insert
    AFTER INSERT OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE site_aggregates_post_insert ();

CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/' || substr(md5(random()::text), 0, 25);
$$;

CREATE TRIGGER person_aggregates_post_count
    AFTER INSERT OR DELETE OR UPDATE OF removed,
    deleted ON post
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_post_count ();

DROP SEQUENCE IF EXISTS changeme_seq;

