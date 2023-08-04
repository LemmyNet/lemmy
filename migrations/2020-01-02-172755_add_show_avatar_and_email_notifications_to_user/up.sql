-- Add columns
ALTER TABLE user_
    ADD COLUMN show_avatars boolean DEFAULT TRUE NOT NULL;

ALTER TABLE user_
    ADD COLUMN send_notifications_to_email boolean DEFAULT FALSE NOT NULL;

-- Rebuild the user_view
DROP VIEW user_view;

CREATE VIEW user_view AS
SELECT
    id,
    name,
    avatar,
    email,
    fedi_name,
    admin,
    banned,
    show_avatars,
    send_notifications_to_email,
    published,
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

