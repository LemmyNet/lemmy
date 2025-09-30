-- Person
-- Drop the 2 views user_alias_1, user_alias_2
DROP VIEW user_alias_1, user_alias_2;

-- rename the user_ table to person
ALTER TABLE user_ RENAME TO person;

ALTER SEQUENCE user__id_seq
    RENAME TO person_id_seq;

-- create a new table local_user
CREATE TABLE local_user (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    password_encrypted text NOT NULL,
    email text UNIQUE,
    admin boolean DEFAULT FALSE NOT NULL,
    show_nsfw boolean DEFAULT FALSE NOT NULL,
    theme character varying(20) DEFAULT 'darkly' ::character varying NOT NULL,
    default_sort_type smallint DEFAULT 0 NOT NULL,
    default_listing_type smallint DEFAULT 1 NOT NULL,
    lang character varying(20) DEFAULT 'browser' ::character varying NOT NULL,
    show_avatars boolean DEFAULT TRUE NOT NULL,
    send_notifications_to_email boolean DEFAULT FALSE NOT NULL,
    matrix_user_id text,
    UNIQUE (person_id)
);

-- Copy the local users over to the new table
INSERT INTO local_user (person_id, password_encrypted, email, admin, show_nsfw, theme, default_sort_type, default_listing_type, lang, show_avatars, send_notifications_to_email, matrix_user_id)
SELECT
    id,
    password_encrypted,
    email,
    admin,
    show_nsfw,
    theme,
    default_sort_type,
    default_listing_type,
    lang,
    show_avatars,
    send_notifications_to_email,
    matrix_user_id
FROM
    person
WHERE
    local = TRUE;

-- Drop those columns from person
ALTER TABLE person
    DROP COLUMN password_encrypted,
    DROP COLUMN email,
    DROP COLUMN admin,
    DROP COLUMN show_nsfw,
    DROP COLUMN theme,
    DROP COLUMN default_sort_type,
    DROP COLUMN default_listing_type,
    DROP COLUMN lang,
    DROP COLUMN show_avatars,
    DROP COLUMN send_notifications_to_email,
    DROP COLUMN matrix_user_id;

-- Rename indexes
ALTER INDEX user__pkey RENAME TO person__pkey;

ALTER INDEX idx_user_actor_id RENAME TO idx_person_actor_id;

ALTER INDEX idx_user_inbox_url RENAME TO idx_person_inbox_url;

ALTER INDEX idx_user_lower_actor_id RENAME TO idx_person_lower_actor_id;

ALTER INDEX idx_user_published RENAME TO idx_person_published;

-- Rename triggers
ALTER TRIGGER site_aggregates_user_delete ON person RENAME TO site_aggregates_person_delete;

ALTER TRIGGER site_aggregates_user_insert ON person RENAME TO site_aggregates_person_insert;

-- Rename the trigger functions
ALTER FUNCTION site_aggregates_user_delete () RENAME TO site_aggregates_person_delete;

ALTER FUNCTION site_aggregates_user_insert () RENAME TO site_aggregates_person_insert;

-- Create views
CREATE VIEW person_alias_1 AS
SELECT
    *
FROM
    person;

CREATE VIEW person_alias_2 AS
SELECT
    *
FROM
    person;

-- Redo user aggregates into person_aggregates
ALTER TABLE user_aggregates RENAME TO person_aggregates;

ALTER SEQUENCE user_aggregates_id_seq
    RENAME TO person_aggregates_id_seq;

ALTER TABLE person_aggregates RENAME COLUMN user_id TO person_id;

-- index
ALTER INDEX user_aggregates_pkey RENAME TO person_aggregates_pkey;

ALTER INDEX idx_user_aggregates_comment_score RENAME TO idx_person_aggregates_comment_score;

ALTER INDEX user_aggregates_user_id_key RENAME TO person_aggregates_person_id_key;

ALTER TABLE person_aggregates RENAME CONSTRAINT user_aggregates_user_id_fkey TO person_aggregates_person_id_fkey;

-- Drop all the old triggers and functions
DROP TRIGGER user_aggregates_user ON person;

DROP TRIGGER user_aggregates_post_count ON post;

DROP TRIGGER user_aggregates_post_score ON post_like;

DROP TRIGGER user_aggregates_comment_count ON comment;

DROP TRIGGER user_aggregates_comment_score ON comment_like;

DROP FUNCTION user_aggregates_user, user_aggregates_post_count, user_aggregates_post_score, user_aggregates_comment_count, user_aggregates_comment_score;

-- initial user add
CREATE FUNCTION person_aggregates_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO person_aggregates (person_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM person_aggregates
        WHERE person_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER person_aggregates_person
    AFTER INSERT OR DELETE ON person
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_person ();

-- post count
CREATE FUNCTION person_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates
        SET
            post_count = post_count - 1
        WHERE
            person_id = OLD.creator_id;
        -- If the post gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            person_aggregates ua
        SET
            post_score = pd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(pl.score)) AS score
                -- User join because posts could be empty
            FROM
                person u
            LEFT JOIN post p ON u.id = p.creator_id
            LEFT JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            u.id) pd
    WHERE
        ua.person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER person_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_post_count ();

-- post score
CREATE FUNCTION person_aggregates_post_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            person_aggregates ua
        SET
            post_score = post_score + NEW.score
        FROM
            post p
        WHERE
            ua.person_id = p.creator_id
            AND p.id = NEW.post_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates ua
        SET
            post_score = post_score - OLD.score
        FROM
            post p
        WHERE
            ua.person_id = p.creator_id
            AND p.id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER person_aggregates_post_score
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_post_score ();

-- comment count
CREATE FUNCTION person_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count + 1
        WHERE
            person_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates
        SET
            comment_count = comment_count - 1
        WHERE
            person_id = OLD.creator_id;
        -- If the comment gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            person_aggregates ua
        SET
            comment_score = cd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(cl.score)) AS score
                -- User join because comments could be empty
            FROM
                person u
            LEFT JOIN comment c ON u.id = c.creator_id
            LEFT JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            u.id) cd
    WHERE
        ua.person_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER person_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_comment_count ();

-- comment score
CREATE FUNCTION person_aggregates_comment_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            person_aggregates ua
        SET
            comment_score = comment_score + NEW.score
        FROM
            comment c
        WHERE
            ua.person_id = c.creator_id
            AND c.id = NEW.comment_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            person_aggregates ua
        SET
            comment_score = comment_score - OLD.score
        FROM
            comment c
        WHERE
            ua.person_id = c.creator_id
            AND c.id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER person_aggregates_comment_score
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE PROCEDURE person_aggregates_comment_score ();

-- person_mention
ALTER TABLE user_mention RENAME TO person_mention;

ALTER SEQUENCE user_mention_id_seq
    RENAME TO person_mention_id_seq;

ALTER INDEX user_mention_pkey RENAME TO person_mention_pkey;

ALTER INDEX user_mention_recipient_id_comment_id_key RENAME TO person_mention_recipient_id_comment_id_key;

ALTER TABLE person_mention RENAME CONSTRAINT user_mention_comment_id_fkey TO person_mention_comment_id_fkey;

ALTER TABLE person_mention RENAME CONSTRAINT user_mention_recipient_id_fkey TO person_mention_recipient_id_fkey;

-- user_ban
ALTER TABLE user_ban RENAME TO person_ban;

ALTER SEQUENCE user_ban_id_seq
    RENAME TO person_ban_id_seq;

ALTER INDEX user_ban_pkey RENAME TO person_ban_pkey;

ALTER INDEX user_ban_user_id_key RENAME TO person_ban_person_id_key;

ALTER TABLE person_ban RENAME COLUMN user_id TO person_id;

ALTER TABLE person_ban RENAME CONSTRAINT user_ban_user_id_fkey TO person_ban_person_id_fkey;

-- comment_like
ALTER TABLE comment_like RENAME COLUMN user_id TO person_id;

ALTER INDEX idx_comment_like_user RENAME TO idx_comment_like_person;

ALTER TABLE comment_like RENAME CONSTRAINT comment_like_comment_id_user_id_key TO comment_like_comment_id_person_id_key;

ALTER TABLE comment_like RENAME CONSTRAINT comment_like_user_id_fkey TO comment_like_person_id_fkey;

-- comment_saved
ALTER TABLE comment_saved RENAME COLUMN user_id TO person_id;

ALTER TABLE comment_saved RENAME CONSTRAINT comment_saved_comment_id_user_id_key TO comment_saved_comment_id_person_id_key;

ALTER TABLE comment_saved RENAME CONSTRAINT comment_saved_user_id_fkey TO comment_saved_person_id_fkey;

-- community_follower
ALTER TABLE community_follower RENAME COLUMN user_id TO person_id;

ALTER TABLE community_follower RENAME CONSTRAINT community_follower_community_id_user_id_key TO community_follower_community_id_person_id_key;

ALTER TABLE community_follower RENAME CONSTRAINT community_follower_user_id_fkey TO community_follower_person_id_fkey;

-- community_moderator
ALTER TABLE community_moderator RENAME COLUMN user_id TO person_id;

ALTER TABLE community_moderator RENAME CONSTRAINT community_moderator_community_id_user_id_key TO community_moderator_community_id_person_id_key;

ALTER TABLE community_moderator RENAME CONSTRAINT community_moderator_user_id_fkey TO community_moderator_person_id_fkey;

-- community_user_ban
ALTER TABLE community_user_ban RENAME TO community_person_ban;

ALTER SEQUENCE community_user_ban_id_seq
    RENAME TO community_person_ban_id_seq;

ALTER TABLE community_person_ban RENAME COLUMN user_id TO person_id;

ALTER TABLE community_person_ban RENAME CONSTRAINT community_user_ban_pkey TO community_person_ban_pkey;

ALTER TABLE community_person_ban RENAME CONSTRAINT community_user_ban_community_id_fkey TO community_person_ban_community_id_fkey;

ALTER TABLE community_person_ban RENAME CONSTRAINT community_user_ban_community_id_user_id_key TO community_person_ban_community_id_person_id_key;

ALTER TABLE community_person_ban RENAME CONSTRAINT community_user_ban_user_id_fkey TO community_person_ban_person_id_fkey;

-- mod_add
ALTER TABLE mod_add RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_add RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_add RENAME CONSTRAINT mod_add_mod_user_id_fkey TO mod_add_mod_person_id_fkey;

ALTER TABLE mod_add RENAME CONSTRAINT mod_add_other_user_id_fkey TO mod_add_other_person_id_fkey;

-- mod_add_community
ALTER TABLE mod_add_community RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_add_community RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_add_community RENAME CONSTRAINT mod_add_community_mod_user_id_fkey TO mod_add_community_mod_person_id_fkey;

ALTER TABLE mod_add_community RENAME CONSTRAINT mod_add_community_other_user_id_fkey TO mod_add_community_other_person_id_fkey;

-- mod_ban
ALTER TABLE mod_ban RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_ban RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_ban RENAME CONSTRAINT mod_ban_mod_user_id_fkey TO mod_ban_mod_person_id_fkey;

ALTER TABLE mod_ban RENAME CONSTRAINT mod_ban_other_user_id_fkey TO mod_ban_other_person_id_fkey;

-- mod_ban_community
ALTER TABLE mod_ban_from_community RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_ban_from_community RENAME COLUMN other_user_id TO other_person_id;

ALTER TABLE mod_ban_from_community RENAME CONSTRAINT mod_ban_from_community_mod_user_id_fkey TO mod_ban_from_community_mod_person_id_fkey;

ALTER TABLE mod_ban_from_community RENAME CONSTRAINT mod_ban_from_community_other_user_id_fkey TO mod_ban_from_community_other_person_id_fkey;

-- mod_lock_post
ALTER TABLE mod_lock_post RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_lock_post RENAME CONSTRAINT mod_lock_post_mod_user_id_fkey TO mod_lock_post_mod_person_id_fkey;

-- mod_remove_comment
ALTER TABLE mod_remove_comment RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_remove_comment RENAME CONSTRAINT mod_remove_comment_mod_user_id_fkey TO mod_remove_comment_mod_person_id_fkey;

-- mod_remove_community
ALTER TABLE mod_remove_community RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_remove_community RENAME CONSTRAINT mod_remove_community_mod_user_id_fkey TO mod_remove_community_mod_person_id_fkey;

-- mod_remove_post
ALTER TABLE mod_remove_post RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_remove_post RENAME CONSTRAINT mod_remove_post_mod_user_id_fkey TO mod_remove_post_mod_person_id_fkey;

-- mod_sticky_post
ALTER TABLE mod_sticky_post RENAME COLUMN mod_user_id TO mod_person_id;

ALTER TABLE mod_sticky_post RENAME CONSTRAINT mod_sticky_post_mod_user_id_fkey TO mod_sticky_post_mod_person_id_fkey;

-- password_reset_request
DELETE FROM password_reset_request;

ALTER TABLE password_reset_request
    DROP COLUMN user_id;

ALTER TABLE password_reset_request
    ADD COLUMN local_user_id integer NOT NULL REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- post_like
ALTER TABLE post_like RENAME COLUMN user_id TO person_id;

ALTER INDEX idx_post_like_user RENAME TO idx_post_like_person;

ALTER TABLE post_like RENAME CONSTRAINT post_like_post_id_user_id_key TO post_like_post_id_person_id_key;

ALTER TABLE post_like RENAME CONSTRAINT post_like_user_id_fkey TO post_like_person_id_fkey;

-- post_read
ALTER TABLE post_read RENAME COLUMN user_id TO person_id;

ALTER TABLE post_read RENAME CONSTRAINT post_read_post_id_user_id_key TO post_read_post_id_person_id_key;

ALTER TABLE post_read RENAME CONSTRAINT post_read_user_id_fkey TO post_read_person_id_fkey;

-- post_saved
ALTER TABLE post_saved RENAME COLUMN user_id TO person_id;

ALTER TABLE post_saved RENAME CONSTRAINT post_saved_post_id_user_id_key TO post_saved_post_id_person_id_key;

ALTER TABLE post_saved RENAME CONSTRAINT post_saved_user_id_fkey TO post_saved_person_id_fkey;

-- redo site aggregates trigger
CREATE OR REPLACE FUNCTION site_aggregates_activity (i text)
    RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    count_ integer;
BEGIN
    SELECT
        count(*) INTO count_
    FROM (
        SELECT
            c.creator_id
        FROM
            comment c
            INNER JOIN person u ON c.creator_id = u.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE
        UNION
        SELECT
            p.creator_id
        FROM
            post p
            INNER JOIN person u ON p.creator_id = u.id
        WHERE
            p.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE) a;
    RETURN count_;
END;
$$;

