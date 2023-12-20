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

CREATE OR REPLACE FUNCTION generate_unique_changeme ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/' || substr(md5(random()::text), 0, 25);
$$;

DROP SEQUENCE IF EXISTS changeme_seq;

