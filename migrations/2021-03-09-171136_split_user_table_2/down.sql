-- post_saved
ALTER TABLE post_saved RENAME COLUMN person_id TO user_id;

ALTER TABLE post_saved RENAME CONSTRAINT post_saved_post_id_person_id_key TO post_saved_post_id_user_id_key;

ALTER TABLE post_saved RENAME CONSTRAINT post_saved_person_id_fkey TO post_saved_user_id_fkey;

-- post_read
ALTER TABLE post_read RENAME COLUMN person_id TO user_id;

ALTER TABLE post_read RENAME CONSTRAINT post_read_post_id_person_id_key TO post_read_post_id_user_id_key;

ALTER TABLE post_read RENAME CONSTRAINT post_read_person_id_fkey TO post_read_user_id_fkey;

-- post_like
ALTER TABLE post_like RENAME COLUMN person_id TO user_id;

ALTER INDEX idx_post_like_person RENAME TO idx_post_like_user;

ALTER TABLE post_like RENAME CONSTRAINT post_like_post_id_person_id_key TO post_like_post_id_user_id_key;

ALTER TABLE post_like RENAME CONSTRAINT post_like_person_id_fkey TO post_like_user_id_fkey;

-- password_reset_request
DELETE FROM password_reset_request;

ALTER TABLE password_reset_request
    DROP COLUMN local_user_id;

ALTER TABLE password_reset_request
    ADD COLUMN user_id integer NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- mod_sticky_post
ALTER TABLE mod_sticky_post RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_sticky_post RENAME CONSTRAINT mod_sticky_post_mod_person_id_fkey TO mod_sticky_post_mod_user_id_fkey;

-- mod_remove_post
ALTER TABLE mod_remove_post RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_remove_post RENAME CONSTRAINT mod_remove_post_mod_person_id_fkey TO mod_remove_post_mod_user_id_fkey;

-- mod_remove_community
ALTER TABLE mod_remove_community RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_remove_community RENAME CONSTRAINT mod_remove_community_mod_person_id_fkey TO mod_remove_community_mod_user_id_fkey;

-- mod_remove_comment
ALTER TABLE mod_remove_comment RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_remove_comment RENAME CONSTRAINT mod_remove_comment_mod_person_id_fkey TO mod_remove_comment_mod_user_id_fkey;

-- mod_lock_post
ALTER TABLE mod_lock_post RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_lock_post RENAME CONSTRAINT mod_lock_post_mod_person_id_fkey TO mod_lock_post_mod_user_id_fkey;

-- mod_add_community
ALTER TABLE mod_ban_from_community RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_ban_from_community RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_ban_from_community RENAME CONSTRAINT mod_ban_from_community_mod_person_id_fkey TO mod_ban_from_community_mod_user_id_fkey;

ALTER TABLE mod_ban_from_community RENAME CONSTRAINT mod_ban_from_community_other_person_id_fkey TO mod_ban_from_community_other_user_id_fkey;

-- mod_ban
ALTER TABLE mod_ban RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_ban RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_ban RENAME CONSTRAINT mod_ban_mod_person_id_fkey TO mod_ban_mod_user_id_fkey;

ALTER TABLE mod_ban RENAME CONSTRAINT mod_ban_other_person_id_fkey TO mod_ban_other_user_id_fkey;

-- mod_add_community
ALTER TABLE mod_add_community RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_add_community RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_add_community RENAME CONSTRAINT mod_add_community_mod_person_id_fkey TO mod_add_community_mod_user_id_fkey;

ALTER TABLE mod_add_community RENAME CONSTRAINT mod_add_community_other_person_id_fkey TO mod_add_community_other_user_id_fkey;

-- mod_add
ALTER TABLE mod_add RENAME COLUMN mod_person_id TO mod_user_id;

ALTER TABLE mod_add RENAME COLUMN other_person_id TO other_user_id;

ALTER TABLE mod_add RENAME CONSTRAINT mod_add_mod_person_id_fkey TO mod_add_mod_user_id_fkey;

ALTER TABLE mod_add RENAME CONSTRAINT mod_add_other_person_id_fkey TO mod_add_other_user_id_fkey;

-- community_user_ban
ALTER TABLE community_person_ban RENAME TO community_user_ban;

ALTER SEQUENCE community_person_ban_id_seq
    RENAME TO community_user_ban_id_seq;

ALTER TABLE community_user_ban RENAME COLUMN person_id TO user_id;

ALTER TABLE community_user_ban RENAME CONSTRAINT community_person_ban_pkey TO community_user_ban_pkey;

ALTER TABLE community_user_ban RENAME CONSTRAINT community_person_ban_community_id_fkey TO community_user_ban_community_id_fkey;

ALTER TABLE community_user_ban RENAME CONSTRAINT community_person_ban_community_id_person_id_key TO community_user_ban_community_id_user_id_key;

ALTER TABLE community_user_ban RENAME CONSTRAINT community_person_ban_person_id_fkey TO community_user_ban_user_id_fkey;

-- community_moderator
ALTER TABLE community_moderator RENAME COLUMN person_id TO user_id;

ALTER TABLE community_moderator RENAME CONSTRAINT community_moderator_community_id_person_id_key TO community_moderator_community_id_user_id_key;

ALTER TABLE community_moderator RENAME CONSTRAINT community_moderator_person_id_fkey TO community_moderator_user_id_fkey;

-- community_follower
ALTER TABLE community_follower RENAME COLUMN person_id TO user_id;

ALTER TABLE community_follower RENAME CONSTRAINT community_follower_community_id_person_id_key TO community_follower_community_id_user_id_key;

ALTER TABLE community_follower RENAME CONSTRAINT community_follower_person_id_fkey TO community_follower_user_id_fkey;

-- comment_saved
ALTER TABLE comment_saved RENAME COLUMN person_id TO user_id;

ALTER TABLE comment_saved RENAME CONSTRAINT comment_saved_comment_id_person_id_key TO comment_saved_comment_id_user_id_key;

ALTER TABLE comment_saved RENAME CONSTRAINT comment_saved_person_id_fkey TO comment_saved_user_id_fkey;

-- comment_like
ALTER TABLE comment_like RENAME COLUMN person_id TO user_id;

ALTER INDEX idx_comment_like_person RENAME TO idx_comment_like_user;

ALTER TABLE comment_like RENAME CONSTRAINT comment_like_comment_id_person_id_key TO comment_like_comment_id_user_id_key;

ALTER TABLE comment_like RENAME CONSTRAINT comment_like_person_id_fkey TO comment_like_user_id_fkey;

-- user_ban
ALTER TABLE person_ban RENAME TO user_ban;

ALTER SEQUENCE person_ban_id_seq
    RENAME TO user_ban_id_seq;

ALTER INDEX person_ban_pkey RENAME TO user_ban_pkey;

ALTER INDEX person_ban_person_id_key RENAME TO user_ban_user_id_key;

ALTER TABLE user_ban RENAME COLUMN person_id TO user_id;

ALTER TABLE user_ban RENAME CONSTRAINT person_ban_person_id_fkey TO user_ban_user_id_fkey;

-- user_mention
ALTER TABLE person_mention RENAME TO user_mention;

ALTER SEQUENCE person_mention_id_seq
    RENAME TO user_mention_id_seq;

ALTER INDEX person_mention_pkey RENAME TO user_mention_pkey;

ALTER INDEX person_mention_recipient_id_comment_id_key RENAME TO user_mention_recipient_id_comment_id_key;

ALTER TABLE user_mention RENAME CONSTRAINT person_mention_comment_id_fkey TO user_mention_comment_id_fkey;

ALTER TABLE user_mention RENAME CONSTRAINT person_mention_recipient_id_fkey TO user_mention_recipient_id_fkey;

-- User aggregates table
ALTER TABLE person_aggregates RENAME TO user_aggregates;

ALTER SEQUENCE person_aggregates_id_seq
    RENAME TO user_aggregates_id_seq;

ALTER TABLE user_aggregates RENAME COLUMN person_id TO user_id;

-- Indexes
ALTER INDEX person_aggregates_pkey RENAME TO user_aggregates_pkey;

ALTER INDEX idx_person_aggregates_comment_score RENAME TO idx_user_aggregates_comment_score;

ALTER INDEX person_aggregates_person_id_key RENAME TO user_aggregates_user_id_key;

ALTER TABLE user_aggregates RENAME CONSTRAINT person_aggregates_person_id_fkey TO user_aggregates_user_id_fkey;

-- Redo the user_aggregates table
DROP TRIGGER person_aggregates_person ON person;

DROP TRIGGER person_aggregates_post_count ON post;

DROP TRIGGER person_aggregates_post_score ON post_like;

DROP TRIGGER person_aggregates_comment_count ON comment;

DROP TRIGGER person_aggregates_comment_score ON comment_like;

DROP FUNCTION person_aggregates_person, person_aggregates_post_count, person_aggregates_post_score, person_aggregates_comment_count, person_aggregates_comment_score;

-- user_ table
-- Drop views
DROP VIEW person_alias_1, person_alias_2;

-- Rename indexes
ALTER INDEX person__pkey RENAME TO user__pkey;

ALTER INDEX idx_person_actor_id RENAME TO idx_user_actor_id;

ALTER INDEX idx_person_inbox_url RENAME TO idx_user_inbox_url;

ALTER INDEX idx_person_lower_actor_id RENAME TO idx_user_lower_actor_id;

ALTER INDEX idx_person_published RENAME TO idx_user_published;

-- Rename triggers
ALTER TRIGGER site_aggregates_person_delete ON person RENAME TO site_aggregates_user_delete;

ALTER TRIGGER site_aggregates_person_insert ON person RENAME TO site_aggregates_user_insert;

-- Rename the trigger functions
ALTER FUNCTION site_aggregates_person_delete () RENAME TO site_aggregates_user_delete;

ALTER FUNCTION site_aggregates_person_insert () RENAME TO site_aggregates_user_insert;

-- Rename the table back to user_
ALTER TABLE person RENAME TO user_;

ALTER SEQUENCE person_id_seq
    RENAME TO user__id_seq;

-- Add the columns back in
ALTER TABLE user_
    ADD COLUMN password_encrypted text NOT NULL DEFAULT 'changeme',
    ADD COLUMN email text UNIQUE,
    ADD COLUMN admin boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN show_nsfw boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN theme character varying(20) DEFAULT 'darkly'::character varying NOT NULL,
    ADD COLUMN default_sort_type smallint DEFAULT 0 NOT NULL,
    ADD COLUMN default_listing_type smallint DEFAULT 1 NOT NULL,
    ADD COLUMN lang character varying(20) DEFAULT 'browser'::character varying NOT NULL,
    ADD COLUMN show_avatars boolean DEFAULT TRUE NOT NULL,
    ADD COLUMN send_notifications_to_email boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN matrix_user_id text UNIQUE;

-- Default is only for existing rows
ALTER TABLE user_
    ALTER COLUMN password_encrypted DROP DEFAULT;

-- Update the user_ table with the local_user data
UPDATE
    user_ u
SET
    password_encrypted = lu.password_encrypted,
    email = lu.email,
    admin = lu.admin,
    show_nsfw = lu.show_nsfw,
    theme = lu.theme,
    default_sort_type = lu.default_sort_type,
    default_listing_type = lu.default_listing_type,
    lang = lu.lang,
    show_avatars = lu.show_avatars,
    send_notifications_to_email = lu.send_notifications_to_email,
    matrix_user_id = lu.matrix_user_id
FROM
    local_user lu
WHERE
    lu.person_id = u.id;

CREATE UNIQUE INDEX idx_user_email_lower ON user_ (lower(email));

CREATE VIEW user_alias_1 AS
SELECT
    id,
    actor_id,
    admin,
    avatar,
    banned,
    banner,
    bio,
    default_listing_type,
    default_sort_type,
    deleted,
    email,
    lang,
    last_refreshed_at,
    local,
    matrix_user_id,
    name,
    password_encrypted,
    preferred_username,
    private_key,
    public_key,
    published,
    send_notifications_to_email,
    show_avatars,
    show_nsfw,
    theme,
    updated
FROM
    user_;

CREATE VIEW user_alias_2 AS
SELECT
    id,
    actor_id,
    admin,
    avatar,
    banned,
    banner,
    bio,
    default_listing_type,
    default_sort_type,
    deleted,
    email,
    lang,
    last_refreshed_at,
    local,
    matrix_user_id,
    name,
    password_encrypted,
    preferred_username,
    private_key,
    public_key,
    published,
    send_notifications_to_email,
    show_avatars,
    show_nsfw,
    theme,
    updated
FROM
    user_;

DROP TABLE local_user;

-- Add the user_aggregates table triggers
-- initial user add
CREATE FUNCTION user_aggregates_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO user_aggregates (user_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM user_aggregates
        WHERE user_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_user
    AFTER INSERT OR DELETE ON user_
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_user ();

-- post count
CREATE FUNCTION user_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            user_aggregates
        SET
            post_count = post_count + 1
        WHERE
            user_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates
        SET
            post_count = post_count - 1
        WHERE
            user_id = OLD.creator_id;
        -- If the post gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            user_aggregates ua
        SET
            post_score = pd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(pl.score)) AS score
                -- User join because posts could be empty
            FROM
                user_ u
            LEFT JOIN post p ON u.id = p.creator_id
            LEFT JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            u.id) pd
    WHERE
        ua.user_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_post_count ();

-- post score
CREATE FUNCTION user_aggregates_post_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            user_aggregates ua
        SET
            post_score = post_score + NEW.score
        FROM
            post p
        WHERE
            ua.user_id = p.creator_id
            AND p.id = NEW.post_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates ua
        SET
            post_score = post_score - OLD.score
        FROM
            post p
        WHERE
            ua.user_id = p.creator_id
            AND p.id = OLD.post_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_post_score
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_post_score ();

-- comment count
CREATE FUNCTION user_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            user_aggregates
        SET
            comment_count = comment_count + 1
        WHERE
            user_id = NEW.creator_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates
        SET
            comment_count = comment_count - 1
        WHERE
            user_id = OLD.creator_id;
        -- If the comment gets deleted, the score calculation trigger won't fire,
        -- so you need to re-calculate
        UPDATE
            user_aggregates ua
        SET
            comment_score = cd.score
        FROM (
            SELECT
                u.id,
                coalesce(0, sum(cl.score)) AS score
                -- User join because comments could be empty
            FROM
                user_ u
            LEFT JOIN comment c ON u.id = c.creator_id
            LEFT JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            u.id) cd
    WHERE
        ua.user_id = OLD.creator_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_comment_count ();

-- comment score
CREATE FUNCTION user_aggregates_comment_score ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        -- Need to get the post creator, not the voter
        UPDATE
            user_aggregates ua
        SET
            comment_score = comment_score + NEW.score
        FROM
            comment c
        WHERE
            ua.user_id = c.creator_id
            AND c.id = NEW.comment_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            user_aggregates ua
        SET
            comment_score = comment_score - OLD.score
        FROM
            comment c
        WHERE
            ua.user_id = c.creator_id
            AND c.id = OLD.comment_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER user_aggregates_comment_score
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE PROCEDURE user_aggregates_comment_score ();

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
            INNER JOIN user_ u ON c.creator_id = u.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE
        UNION
        SELECT
            p.creator_id
        FROM
            post p
            INNER JOIN user_ u ON p.creator_id = u.id
        WHERE
            p.published > ('now'::timestamp - i::interval)
            AND u.local = TRUE) a;
    RETURN count_;
END;
$$;

