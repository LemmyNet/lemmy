-- Drop the dependent views
DROP VIEW post_view;

DROP VIEW post_mview;

DROP MATERIALIZED VIEW post_aggregates_mview;

DROP VIEW post_aggregates_view;

DROP VIEW mod_remove_post_view;

DROP VIEW mod_sticky_post_view;

DROP VIEW mod_lock_post_view;

DROP VIEW mod_remove_comment_view;

ALTER TABLE post
    ALTER COLUMN name TYPE varchar(100);

-- regen post view
CREATE VIEW post_aggregates_view AS
SELECT
    p.*,
    (
        SELECT
            u.banned
        FROM
            user_ u
        WHERE
            p.creator_id = u.id) AS banned,
    (
        SELECT
            cb.id::bool
        FROM
            community_user_ban cb
        WHERE
            p.creator_id = cb.user_id
            AND p.community_id = cb.community_id) AS banned_from_community,
    (
        SELECT
            name
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_name,
    (
        SELECT
            avatar
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_avatar,
    (
        SELECT
            name
        FROM
            community
        WHERE
            p.community_id = community.id) AS community_name,
    (
        SELECT
            removed
        FROM
            community c
        WHERE
            p.community_id = c.id) AS community_removed,
    (
        SELECT
            deleted
        FROM
            community c
        WHERE
            p.community_id = c.id) AS community_deleted,
    (
        SELECT
            nsfw
        FROM
            community c
        WHERE
            p.community_id = c.id) AS community_nsfw,
    (
        SELECT
            count(*)
        FROM
            comment
        WHERE
            comment.post_id = p.id) AS number_of_comments,
    coalesce(sum(pl.score), 0) AS score,
    count(
        CASE WHEN pl.score = 1 THEN
            1
        ELSE
            NULL
        END) AS upvotes,
    count(
        CASE WHEN pl.score = - 1 THEN
            1
        ELSE
            NULL
        END) AS downvotes,
    hot_rank (coalesce(sum(pl.score), 0), p.published) AS hot_rank
FROM
    post p
    LEFT JOIN post_like pl ON p.id = pl.post_id
GROUP BY
    p.id;

CREATE MATERIALIZED VIEW post_aggregates_mview AS
SELECT
    *
FROM
    post_aggregates_view;

CREATE UNIQUE INDEX idx_post_aggregates_mview_id ON post_aggregates_mview (id);

CREATE VIEW post_view AS
with all_post AS (
    SELECT
        pa.*
    FROM
        post_aggregates_view pa
)
SELECT
    ap.*,
    u.id AS user_id,
    coalesce(pl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::bool
        FROM
            community_follower cf
        WHERE
            u.id = cf.user_id
            AND cf.community_id = ap.community_id) AS subscribed,
    (
        SELECT
            pr.id::bool
        FROM
            post_read pr
        WHERE
            u.id = pr.user_id
            AND pr.post_id = ap.id) AS read,
    (
        SELECT
            ps.id::bool
        FROM
            post_saved ps
        WHERE
            u.id = ps.user_id
            AND ps.post_id = ap.id) AS saved
FROM
    user_ u
    CROSS JOIN all_post ap
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND ap.id = pl.post_id
    UNION ALL
    SELECT
        ap.*,
        NULL AS user_id,
        NULL AS my_vote,
        NULL AS subscribed,
        NULL AS read,
        NULL AS saved
    FROM
        all_post ap;

CREATE VIEW post_mview AS
with all_post AS (
    SELECT
        pa.*
    FROM
        post_aggregates_mview pa
)
SELECT
    ap.*,
    u.id AS user_id,
    coalesce(pl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::bool
        FROM
            community_follower cf
        WHERE
            u.id = cf.user_id
            AND cf.community_id = ap.community_id) AS subscribed,
    (
        SELECT
            pr.id::bool
        FROM
            post_read pr
        WHERE
            u.id = pr.user_id
            AND pr.post_id = ap.id) AS read,
    (
        SELECT
            ps.id::bool
        FROM
            post_saved ps
        WHERE
            u.id = ps.user_id
            AND ps.post_id = ap.id) AS saved
FROM
    user_ u
    CROSS JOIN all_post ap
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND ap.id = pl.post_id
    UNION ALL
    SELECT
        ap.*,
        NULL AS user_id,
        NULL AS my_vote,
        NULL AS subscribed,
        NULL AS read,
        NULL AS saved
    FROM
        all_post ap;

-- The mod views
CREATE VIEW mod_remove_post_view AS
SELECT
    mrp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            mrp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            community c
        WHERE
            mrp.post_id = p.id
            AND p.community_id = c.id) AS community_id,
    (
        SELECT
            c.name
        FROM
            post p,
            community c
        WHERE
            mrp.post_id = p.id
            AND p.community_id = c.id) AS community_name
FROM
    mod_remove_post mrp;

CREATE VIEW mod_lock_post_view AS
SELECT
    mlp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mlp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            mlp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            community c
        WHERE
            mlp.post_id = p.id
            AND p.community_id = c.id) AS community_id,
    (
        SELECT
            c.name
        FROM
            post p,
            community c
        WHERE
            mlp.post_id = p.id
            AND p.community_id = c.id) AS community_name
FROM
    mod_lock_post mlp;

CREATE VIEW mod_remove_comment_view AS
SELECT
    mrc.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrc.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            c.id
        FROM
            comment c
        WHERE
            mrc.comment_id = c.id) AS comment_user_id,
    (
        SELECT
            name
        FROM
            user_ u,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND u.id = c.creator_id) AS comment_user_name,
    (
        SELECT
            content
        FROM
            comment c
        WHERE
            mrc.comment_id = c.id) AS comment_content,
    (
        SELECT
            p.id
        FROM
            post p,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id) AS post_id,
    (
        SELECT
            p.name
        FROM
            post p,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id) AS post_name,
    (
        SELECT
            co.id
        FROM
            comment c,
            post p,
            community co
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id
            AND p.community_id = co.id) AS community_id,
    (
        SELECT
            co.name
        FROM
            comment c,
            post p,
            community co
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id
            AND p.community_id = co.id) AS community_name
FROM
    mod_remove_comment mrc;

CREATE VIEW mod_sticky_post_view AS
SELECT
    msp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            msp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            msp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            community c
        WHERE
            msp.post_id = p.id
            AND p.community_id = c.id) AS community_id,
    (
        SELECT
            c.name
        FROM
            post p,
            community c
        WHERE
            msp.post_id = p.id
            AND p.community_id = c.id) AS community_name
FROM
    mod_sticky_post msp;

