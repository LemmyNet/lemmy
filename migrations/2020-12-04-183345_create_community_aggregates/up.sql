-- Add community aggregates
CREATE TABLE community_aggregates (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    subscribers bigint NOT NULL DEFAULT 0,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (community_id)
);

INSERT INTO community_aggregates (community_id, subscribers, posts, comments, published)
SELECT
    c.id,
    coalesce(cf.subs, 0) AS subscribers,
    coalesce(cd.posts, 0) AS posts,
    coalesce(cd.comments, 0) AS comments,
    c.published
FROM
    community c
    LEFT JOIN (
        SELECT
            p.community_id,
            count(DISTINCT p.id) AS posts,
            count(DISTINCT ct.id) AS comments
        FROM
            post p
            LEFT JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            p.community_id) cd ON cd.community_id = c.id
    LEFT JOIN (
        SELECT
            community_follower.community_id,
            count(*) AS subs
        FROM
            community_follower
        GROUP BY
            community_follower.community_id) cf ON cf.community_id = c.id;

-- Add community aggregate triggers
-- initial community add
CREATE FUNCTION community_aggregates_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO community_aggregates (community_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM community_aggregates
        WHERE community_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER community_aggregates_community
    AFTER INSERT OR DELETE ON community
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_community ();

-- post count
CREATE FUNCTION community_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates
        SET
            posts = posts + 1
        WHERE
            community_id = NEW.community_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates
        SET
            posts = posts - 1
        WHERE
            community_id = OLD.community_id;
        -- Update the counts if the post got deleted
        UPDATE
            community_aggregates ca
        SET
            posts = coalesce(cd.posts, 0),
            comments = coalesce(cd.comments, 0)
        FROM (
            SELECT
                c.id,
                count(DISTINCT p.id) AS posts,
                count(DISTINCT ct.id) AS comments
            FROM
                community c
            LEFT JOIN post p ON c.id = p.community_id
            LEFT JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            c.id) cd
    WHERE
        ca.community_id = OLD.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER community_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_post_count ();

-- comment count
CREATE FUNCTION community_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments + 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = NEW.post_id
            AND ca.community_id = p.community_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates ca
        SET
            comments = comments - 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = OLD.post_id
            AND ca.community_id = p.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER community_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_comment_count ();

-- subscriber count
CREATE FUNCTION community_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            community_aggregates
        SET
            subscribers = subscribers + 1
        WHERE
            community_id = NEW.community_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            community_aggregates
        SET
            subscribers = subscribers - 1
        WHERE
            community_id = OLD.community_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER community_aggregates_subscriber_count
    AFTER INSERT OR DELETE ON community_follower
    FOR EACH ROW
    EXECUTE PROCEDURE community_aggregates_subscriber_count ();

