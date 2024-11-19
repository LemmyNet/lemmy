CREATE OR REPLACE FUNCTION refresh_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM community_aggregates_fast
        WHERE id = OLD.id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM community_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO community_aggregates_fast
        SELECT
            *
        FROM
            community_aggregates_view
        WHERE
            id = NEW.id;
        -- Update user view due to owner changes
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id;
        -- Update post view due to community changes
        DELETE FROM post_aggregates_fast
        WHERE community_id = NEW.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            community_id = NEW.id;
        -- TODO make sure this shows up in the users page ?
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO community_aggregates_fast
        SELECT
            *
        FROM
            community_aggregates_view
        WHERE
            id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION refresh_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM user_fast
        WHERE id = OLD.id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM user_fast
        WHERE id = OLD.id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.id;
        -- Refresh post_fast, cause of user info changes
        DELETE FROM post_aggregates_fast
        WHERE creator_id = NEW.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            creator_id = NEW.id;
        DELETE FROM comment_aggregates_fast
        WHERE creator_id = NEW.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            creator_id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION refresh_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        -- Update community number of posts
        UPDATE
            community_aggregates_fast
        SET
            number_of_posts = number_of_posts - 1
        WHERE
            id = OLD.community_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
        -- Update that users number of posts, post score
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id;
        -- Update community number of posts
        UPDATE
            community_aggregates_fast
        SET
            number_of_posts = number_of_posts + 1
        WHERE
            id = NEW.community_id;
        -- Update the hot rank on the post table
        -- TODO this might not correctly update it, using a 1 week interval
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = pav.hot_rank
        FROM
            post_aggregates_view AS pav
        WHERE
            paf.id = pav.id
            AND (pav.published > ('now'::timestamp - '1 week'::interval));
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION refresh_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        -- Update community number of comments
        UPDATE
            community_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments - 1
        FROM
            post AS p
        WHERE
            caf.id = p.community_id
            AND p.id = OLD.post_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
        -- Update user view due to comment count
        UPDATE
            user_fast
        SET
            number_of_comments = number_of_comments + 1
        WHERE
            id = NEW.creator_id;
        -- Update post view due to comment count, new comment activity time, but only on new posts
        -- TODO this could be done more efficiently
        DELETE FROM post_aggregates_fast
        WHERE id = NEW.post_id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.post_id;
        -- Force the hot rank as zero on week-older posts
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = 0
        WHERE
            paf.id = NEW.post_id
            AND (paf.published < ('now'::timestamp - '1 week'::interval));
        -- Update community number of comments
        UPDATE
            community_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments + 1
        FROM
            post AS p
        WHERE
            caf.id = p.community_id
            AND p.id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

