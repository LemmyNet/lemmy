-- Drop the triggers
DROP TRIGGER refresh_private_message ON private_message;

DROP FUNCTION refresh_private_message ();

-- Drop the view and table
DROP VIEW private_message_view CASCADE;

DROP TABLE private_message;

-- Rebuild the old views
DROP VIEW user_view CASCADE;

CREATE VIEW user_view AS
SELECT
    u.id,
    u.name,
    u.avatar,
    u.email,
    u.fedi_name,
    u.admin,
    u.banned,
    u.show_avatars,
    u.send_notifications_to_email,
    u.published,
    (
        SELECT
            count(*)
        FROM
            post p
        WHERE
            p.creator_id = u.id) AS number_of_posts,
    (
        SELECT
            coalesce(sum(score), 0)
        FROM
            post p,
            post_like pl
        WHERE
            u.id = p.creator_id
            AND p.id = pl.post_id) AS post_score,
    (
        SELECT
            count(*)
        FROM
            comment c
        WHERE
            c.creator_id = u.id) AS number_of_comments,
    (
        SELECT
            coalesce(sum(score), 0)
        FROM
            comment c,
            comment_like cl
        WHERE
            u.id = c.creator_id
            AND c.id = cl.comment_id) AS comment_score
FROM
    user_ u;

CREATE MATERIALIZED VIEW user_mview AS
SELECT
    *
FROM
    user_view;

CREATE UNIQUE INDEX idx_user_mview_id ON user_mview (id);

-- Drop the columns
ALTER TABLE user_
    DROP COLUMN matrix_user_id;

