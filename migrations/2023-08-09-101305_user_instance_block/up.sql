CREATE TABLE instance_block (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (person_id, instance_id)
);

ALTER TABLE post_aggregates
    ADD COLUMN instance_id integer REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

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

UPDATE
    post_aggregates
SET
    instance_id = community.instance_id
FROM
    post
    JOIN community ON post.community_id = community.id
WHERE
    post.id = post_aggregates.post_id;

ALTER TABLE post_aggregates
    ALTER COLUMN instance_id SET NOT NULL;

