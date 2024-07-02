-- functions and triggers
DROP TRIGGER IF EXISTS refresh_user ON user_;

DROP FUNCTION IF EXISTS refresh_user ();

DROP TRIGGER IF EXISTS refresh_post ON post;

DROP FUNCTION IF EXISTS refresh_post ();

DROP TRIGGER IF EXISTS refresh_post_like ON post_like;

DROP FUNCTION IF EXISTS refresh_post_like ();

DROP TRIGGER IF EXISTS refresh_community ON community;

DROP FUNCTION IF EXISTS refresh_community ();

DROP TRIGGER IF EXISTS refresh_community_follower ON community_follower;

DROP FUNCTION IF EXISTS refresh_community_follower ();

DROP TRIGGER IF EXISTS refresh_community_user_ban ON community_user_ban;

DROP FUNCTION IF EXISTS refresh_community_user_ban ();

DROP TRIGGER IF EXISTS refresh_comment ON comment;

DROP FUNCTION IF EXISTS refresh_comment ();

DROP TRIGGER IF EXISTS refresh_comment_like ON comment_like;

DROP FUNCTION IF EXISTS refresh_comment_like ();

-- post
-- Recreate the view
DROP VIEW IF EXISTS post_view;

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

DROP VIEW IF EXISTS post_mview;

DROP MATERIALIZED VIEW post_aggregates_mview;

DROP VIEW IF EXISTS post_aggregates_view;

-- user
DROP MATERIALIZED VIEW user_mview;

DROP VIEW IF EXISTS user_view;

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

-- community
DROP VIEW IF EXISTS community_mview;

DROP MATERIALIZED VIEW community_aggregates_mview;

DROP VIEW IF EXISTS community_view;

DROP VIEW IF EXISTS community_aggregates_view;

CREATE VIEW community_view AS
with all_community AS (
    SELECT
        *,
        (
            SELECT
                name
            FROM
                user_ u
            WHERE
                c.creator_id = u.id) AS creator_name,
        (
            SELECT
                avatar
            FROM
                user_ u
            WHERE
                c.creator_id = u.id) AS creator_avatar,
        (
            SELECT
                name
            FROM
                category ct
            WHERE
                c.category_id = ct.id) AS category_name,
        (
            SELECT
                count(*)
            FROM
                community_follower cf
            WHERE
                cf.community_id = c.id) AS number_of_subscribers,
        (
            SELECT
                count(*)
            FROM
                post p
            WHERE
                p.community_id = c.id) AS number_of_posts,
        (
            SELECT
                count(*)
            FROM
                comment co,
                post p
            WHERE
                c.id = p.community_id
                AND p.id = co.post_id) AS number_of_comments,
        hot_rank ((
            SELECT
                count(*)
            FROM community_follower cf
            WHERE
                cf.community_id = c.id), c.published) AS hot_rank
FROM
    community c
)
SELECT
    ac.*,
    u.id AS user_id,
    (
        SELECT
            cf.id::boolean
        FROM
            community_follower cf
        WHERE
            u.id = cf.user_id
            AND ac.id = cf.community_id) AS subscribed
FROM
    user_ u
    CROSS JOIN all_community ac
UNION ALL
SELECT
    ac.*,
    NULL AS user_id,
    NULL AS subscribed
FROM
    all_community ac;

-- reply and comment view
DROP VIEW IF EXISTS reply_view;

DROP VIEW IF EXISTS user_mention_view;

DROP VIEW IF EXISTS comment_view;

DROP VIEW IF EXISTS comment_mview;

DROP MATERIALIZED VIEW comment_aggregates_mview;

DROP VIEW IF EXISTS comment_aggregates_view;

CREATE VIEW comment_view AS
with all_comment AS (
    SELECT
        c.*,
        (
            SELECT
                community_id
            FROM
                post p
            WHERE
                p.id = c.post_id),
            (
                SELECT
                    u.banned
                FROM
                    user_ u
                WHERE
                    c.creator_id = u.id) AS banned,
                (
                    SELECT
                        cb.id::bool
                    FROM
                        community_user_ban cb,
                        post p
                    WHERE
                        c.creator_id = cb.user_id
                        AND p.id = c.post_id
                        AND p.community_id = cb.community_id) AS banned_from_community,
                    (
                        SELECT
                            name
                        FROM
                            user_
                        WHERE
                            c.creator_id = user_.id) AS creator_name,
                        (
                            SELECT
                                avatar
                            FROM
                                user_
                            WHERE
                                c.creator_id = user_.id) AS creator_avatar,
                            coalesce(sum(cl.score), 0) AS score,
                        count(
                            CASE WHEN cl.score = 1 THEN
                                1
                            ELSE
                                NULL
                            END) AS upvotes,
                        count(
                            CASE WHEN cl.score = - 1 THEN
                                1
                            ELSE
                                NULL
                            END) AS downvotes
                    FROM
                        comment c
                    LEFT JOIN comment_like cl ON c.id = cl.comment_id
                GROUP BY
                    c.id
)
        SELECT
            ac.*,
            u.id AS user_id,
            coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cs.id::bool
        FROM
            comment_saved cs
        WHERE
            u.id = cs.user_id
            AND cs.comment_id = ac.id) AS saved
FROM
    user_ u
    CROSS JOIN all_comment ac
    LEFT JOIN comment_like cl ON u.id = cl.user_id
        AND ac.id = cl.comment_id
    UNION ALL
    SELECT
        ac.*,
        NULL AS user_id,
        NULL AS my_vote,
        NULL AS saved
    FROM
        all_comment ac;

CREATE VIEW reply_view AS
with closereply AS (
    SELECT
        c2.id,
        c2.creator_id AS sender_id,
        c.creator_id AS recipient_id
    FROM
        comment c
        INNER JOIN comment c2 ON c.id = c2.parent_id
    WHERE
        c2.creator_id != c.creator_id
        -- Do union where post is null
    UNION
    SELECT
        c.id,
        c.creator_id AS sender_id,
        p.creator_id AS recipient_id
    FROM
        comment c,
        post p
    WHERE
        c.post_id = p.id
        AND c.parent_id IS NULL
        AND c.creator_id != p.creator_id
)
SELECT
    cv.*,
    closereply.recipient_id
FROM
    comment_view cv,
    closereply
WHERE
    closereply.id = cv.id;

-- user mention
CREATE VIEW user_mention_view AS
SELECT
    c.id,
    um.id AS user_mention_id,
    c.creator_id,
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.community_id,
    c.banned,
    c.banned_from_community,
    c.creator_name,
    c.creator_avatar,
    c.score,
    c.upvotes,
    c.downvotes,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id
FROM
    user_mention um,
    comment_view c
WHERE
    um.comment_id = c.id;

