-- post
DROP VIEW post_view;

CREATE MATERIALIZED VIEW post_view AS
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
        all_post ap WITH data;

CREATE UNIQUE INDEX idx_post_view_unique ON post_view (id, user_id);

CREATE INDEX idx_post_view_user_id ON post_view (user_id);

CREATE INDEX idx_post_view_hot_rank_published ON post_view (hot_rank DESC, published DESC);

CREATE INDEX idx_post_view_published ON post_view (published DESC);

CREATE INDEX idx_post_view_score ON post_view (score DESC);

-- user_view
DROP VIEW user_view;

CREATE MATERIALIZED VIEW user_view AS
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

CREATE UNIQUE INDEX idx_user_view_unique ON user_view (id);

CREATE INDEX idx_user_view_comment_published ON user_view (comment_score DESC, published DESC);

CREATE INDEX idx_user_view_admin ON user_view (admin);

CREATE INDEX idx_user_view_banned ON user_view (banned);

-- community
DROP VIEW community_view;

CREATE MATERIALIZED VIEW community_view AS
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

CREATE UNIQUE INDEX idx_community_view_unique ON community_view (id, user_id);

CREATE INDEX idx_community_view_user_id ON community_view (user_id);

CREATE INDEX idx_community_view_hot_rank_subscribed ON community_view (hot_rank DESC, number_of_subscribers DESC);

-- reply and comment view
DROP VIEW reply_view;

DROP VIEW user_mention_view;

DROP VIEW comment_view;

CREATE MATERIALIZED VIEW comment_view AS
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

CREATE UNIQUE INDEX idx_comment_view_unique ON comment_view (id, user_id);

CREATE INDEX idx_comment_view_user_id ON comment_view (user_id);

CREATE INDEX idx_comment_view_creator_id ON comment_view (creator_id);

CREATE INDEX idx_comment_view_post_id ON comment_view (post_id);

CREATE INDEX idx_comment_view_score ON comment_view (score DESC);

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

-- user
CREATE OR REPLACE FUNCTION refresh_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_view;
    -- cause of bans
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_user
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON user_
    FOR EACH statement
    EXECUTE PROCEDURE refresh_user ();

-- post
CREATE OR REPLACE FUNCTION refresh_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_post
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON post
    FOR EACH statement
    EXECUTE PROCEDURE refresh_post ();

-- post_like
CREATE OR REPLACE FUNCTION refresh_post_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_post_like
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON post_like
    FOR EACH statement
    EXECUTE PROCEDURE refresh_post_like ();

-- community
CREATE OR REPLACE FUNCTION refresh_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_view;
    REFRESH MATERIALIZED VIEW CONCURRENTLY community_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_community
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON community
    FOR EACH statement
    EXECUTE PROCEDURE refresh_community ();

-- community_follower
CREATE OR REPLACE FUNCTION refresh_community_follower ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY community_view;
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_community_follower
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON community_follower
    FOR EACH statement
    EXECUTE PROCEDURE refresh_community_follower ();

-- comment
CREATE OR REPLACE FUNCTION refresh_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_view;
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_comment
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON comment
    FOR EACH statement
    EXECUTE PROCEDURE refresh_comment ();

-- comment_like
CREATE OR REPLACE FUNCTION refresh_comment_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_view;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_comment_like
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON comment_like
    FOR EACH statement
    EXECUTE PROCEDURE refresh_comment_like ();

