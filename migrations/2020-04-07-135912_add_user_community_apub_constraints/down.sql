-- User table
DROP VIEW user_view CASCADE;

ALTER TABLE user_
    ADD COLUMN fedi_name varchar(40) NOT NULL DEFAULT 'http://fake.com';

ALTER TABLE user_
    ADD CONSTRAINT user__name_fedi_name_key UNIQUE (name, fedi_name);

-- Community
ALTER TABLE community
    ADD CONSTRAINT community_name_key UNIQUE (name);

CREATE VIEW user_view AS
SELECT
    u.id,
    u.name,
    u.avatar,
    u.email,
    u.matrix_user_id,
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

