-- Add the column
ALTER TABLE post
    ADD COLUMN stickied boolean DEFAULT FALSE NOT NULL;

-- Add the mod table
CREATE TABLE mod_sticky_post (
    id serial PRIMARY KEY,
    mod_user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    stickied boolean DEFAULT TRUE,
    when_ timestamp NOT NULL DEFAULT now()
);

-- Add mod view
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

-- Recreate the view
DROP VIEW post_view;

CREATE VIEW post_view AS
with all_post AS (
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
        p.id
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

