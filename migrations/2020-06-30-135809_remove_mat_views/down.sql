-- Dropping all the fast tables
DROP TABLE user_fast;

DROP VIEW post_fast_view;

DROP TABLE post_aggregates_fast;

DROP VIEW community_fast_view;

DROP TABLE community_aggregates_fast;

DROP VIEW reply_fast_view;

DROP VIEW user_mention_fast_view;

DROP VIEW comment_fast_view;

DROP TABLE comment_aggregates_fast;

-- Re-adding all the triggers, functions, and mviews
-- private message
CREATE MATERIALIZED VIEW private_message_mview AS
SELECT
    *
FROM
    private_message_view;

CREATE UNIQUE INDEX idx_private_message_mview_id ON private_message_mview (id);

-- Create the triggers
CREATE OR REPLACE FUNCTION refresh_private_message ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY private_message_mview;
    RETURN NULL;
END
$$;

CREATE TRIGGER refresh_private_message
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON private_message
    FOR EACH statement
    EXECUTE PROCEDURE refresh_private_message ();

-- user
CREATE OR REPLACE FUNCTION refresh_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_aggregates_mview;
    -- cause of bans
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    RETURN NULL;
END
$$;

DROP TRIGGER refresh_user ON user_;

CREATE TRIGGER refresh_user
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON user_
    FOR EACH statement
    EXECUTE PROCEDURE refresh_user ();

DROP VIEW user_view CASCADE;

CREATE VIEW user_view AS
SELECT
    u.id,
    u.actor_id,
    u.name,
    u.avatar,
    u.email,
    u.matrix_user_id,
    u.bio,
    u.local,
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

-- community
DROP TRIGGER refresh_community ON community;

CREATE TRIGGER refresh_community
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON community
    FOR EACH statement
    EXECUTE PROCEDURE refresh_community ();

CREATE OR REPLACE FUNCTION refresh_community ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY community_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_mview;
    RETURN NULL;
END
$$;

DROP VIEW community_aggregates_view CASCADE;

CREATE VIEW community_aggregates_view AS
-- Now that there's public and private keys, you have to be explicit here
SELECT
    c.id,
    c.name,
    c.title,
    c.description,
    c.category_id,
    c.creator_id,
    c.removed,
    c.published,
    c.updated,
    c.deleted,
    c.nsfw,
    c.actor_id,
    c.local,
    c.last_refreshed_at,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            c.creator_id = u.id) AS creator_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            c.creator_id = u.id) AS creator_local,
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
    community c;

CREATE MATERIALIZED VIEW community_aggregates_mview AS
SELECT
    *
FROM
    community_aggregates_view;

CREATE UNIQUE INDEX idx_community_aggregates_mview_id ON community_aggregates_mview (id);

CREATE VIEW community_view AS
with all_community AS (
    SELECT
        ca.*
    FROM
        community_aggregates_view ca
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

CREATE VIEW community_mview AS
with all_community AS (
    SELECT
        ca.*
    FROM
        community_aggregates_mview ca
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

-- Post
DROP VIEW post_view;

DROP VIEW post_aggregates_view;

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
            actor_id
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_actor_id,
    (
        SELECT
            local
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_local,
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
            actor_id
        FROM
            community
        WHERE
            p.community_id = community.id) AS community_actor_id,
    (
        SELECT
            local
        FROM
            community
        WHERE
            p.community_id = community.id) AS community_local,
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
    hot_rank (coalesce(sum(pl.score), 0), (
            CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
                p.published -- Prevents necro-bumps
            ELSE
                greatest (c.recent_comment_time, p.published)
            END)) AS hot_rank,
    (
        CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
            p.published -- Prevents necro-bumps
        ELSE
            greatest (c.recent_comment_time, p.published)
        END) AS newest_activity_time
FROM
    post p
    LEFT JOIN post_like pl ON p.id = pl.post_id
    LEFT JOIN (
        SELECT
            post_id,
            max(published) AS recent_comment_time
        FROM
            comment
        GROUP BY
            1) c ON p.id = c.post_id
GROUP BY
    p.id,
    c.recent_comment_time;

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

DROP TRIGGER refresh_post ON post;

CREATE TRIGGER refresh_post
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON post
    FOR EACH statement
    EXECUTE PROCEDURE refresh_post ();

CREATE OR REPLACE FUNCTION refresh_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_mview;
    RETURN NULL;
END
$$;

-- User mention, comment, reply
DROP VIEW user_mention_view;

DROP VIEW comment_view;

DROP VIEW comment_aggregates_view;

-- reply and comment view
CREATE VIEW comment_aggregates_view AS
SELECT
    c.*,
    (
        SELECT
            community_id
        FROM
            post p
        WHERE
            p.id = c.post_id), (
        SELECT
            co.actor_id
        FROM
            post p,
            community co
        WHERE
            p.id = c.post_id
            AND p.community_id = co.id) AS community_actor_id,
    (
        SELECT
            co.local
        FROM
            post p,
            community co
        WHERE
            p.id = c.post_id
            AND p.community_id = co.id) AS community_local,
    (
        SELECT
            co.name
        FROM
            post p,
            community co
        WHERE
            p.id = c.post_id
            AND p.community_id = co.id) AS community_name,
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
            actor_id
        FROM
            user_
        WHERE
            c.creator_id = user_.id) AS creator_actor_id,
    (
        SELECT
            local
        FROM
            user_
        WHERE
            c.creator_id = user_.id) AS creator_local,
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
        END) AS downvotes,
    hot_rank (coalesce(sum(cl.score), 0), c.published) AS hot_rank
FROM
    comment c
    LEFT JOIN comment_like cl ON c.id = cl.comment_id
GROUP BY
    c.id;

CREATE MATERIALIZED VIEW comment_aggregates_mview AS
SELECT
    *
FROM
    comment_aggregates_view;

CREATE UNIQUE INDEX idx_comment_aggregates_mview_id ON comment_aggregates_mview (id);

CREATE VIEW comment_view AS
with all_comment AS (
    SELECT
        ca.*
    FROM
        comment_aggregates_view ca
)
SELECT
    ac.*,
    u.id AS user_id,
    coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::boolean
        FROM
            community_follower cf
        WHERE
            u.id = cf.user_id
            AND ac.community_id = cf.community_id) AS subscribed,
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
        NULL AS subscribed,
        NULL AS saved
    FROM
        all_comment ac;

CREATE VIEW comment_mview AS
with all_comment AS (
    SELECT
        ca.*
    FROM
        comment_aggregates_mview ca
)
SELECT
    ac.*,
    u.id AS user_id,
    coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::boolean
        FROM
            community_follower cf
        WHERE
            u.id = cf.user_id
            AND ac.community_id = cf.community_id) AS subscribed,
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
        NULL AS subscribed,
        NULL AS saved
    FROM
        all_comment ac;

-- Do the reply_view referencing the comment_mview
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
    comment_mview cv,
    closereply
WHERE
    closereply.id = cv.id;

-- user mention
CREATE VIEW user_mention_view AS
SELECT
    c.id,
    um.id AS user_mention_id,
    c.creator_id,
    c.creator_actor_id,
    c.creator_local,
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.community_id,
    c.community_actor_id,
    c.community_local,
    c.community_name,
    c.banned,
    c.banned_from_community,
    c.creator_name,
    c.creator_avatar,
    c.score,
    c.upvotes,
    c.downvotes,
    c.hot_rank,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_local
FROM
    user_mention um,
    comment_view c
WHERE
    um.comment_id = c.id;

CREATE VIEW user_mention_mview AS
with all_comment AS (
    SELECT
        ca.*
    FROM
        comment_aggregates_mview ca
)
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    u.id AS user_id,
    coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cs.id::bool
        FROM
            comment_saved cs
        WHERE
            u.id = cs.user_id
            AND cs.comment_id = ac.id) AS saved,
    um.recipient_id,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_local
FROM
    user_ u
    CROSS JOIN all_comment ac
    LEFT JOIN comment_like cl ON u.id = cl.user_id
        AND ac.id = cl.comment_id
    LEFT JOIN user_mention um ON um.comment_id = ac.id
UNION ALL
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS saved,
    um.recipient_id,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_local
FROM
    all_comment ac
    LEFT JOIN user_mention um ON um.comment_id = ac.id;

-- comment
CREATE OR REPLACE FUNCTION refresh_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY community_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_mview;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER refresh_comment
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON comment
    FOR EACH statement
    EXECUTE PROCEDURE refresh_comment ();

-- comment_like
CREATE OR REPLACE FUNCTION refresh_comment_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_mview;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER refresh_comment_like
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON comment_like
    FOR EACH statement
    EXECUTE PROCEDURE refresh_comment_like ();

-- community_follower
CREATE OR REPLACE FUNCTION refresh_community_follower ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY community_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER refresh_community_follower
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON community_follower
    FOR EACH statement
    EXECUTE PROCEDURE refresh_community_follower ();

-- community_user_ban
CREATE OR REPLACE FUNCTION refresh_community_user_ban ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY comment_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER refresh_community_user_ban
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON community_user_ban
    FOR EACH statement
    EXECUTE PROCEDURE refresh_community_user_ban ();

-- post_like
CREATE OR REPLACE FUNCTION refresh_post_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY post_aggregates_mview;
    REFRESH MATERIALIZED VIEW CONCURRENTLY user_mview;
    RETURN NULL;
END
$$;

CREATE OR REPLACE TRIGGER refresh_post_like
    AFTER INSERT OR UPDATE OR DELETE OR TRUNCATE ON post_like
    FOR EACH statement
    EXECUTE PROCEDURE refresh_post_like ();

CREATE OR REPLACE VIEW community_moderator_view AS
SELECT
    *,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            cm.user_id = u.id), (
        SELECT
            actor_id
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_actor_id,
    (
        SELECT
            local
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_local,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_name
FROM
    community_moderator cm;

CREATE OR REPLACE VIEW community_follower_view AS
SELECT
    *,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            cf.user_id = u.id), (
        SELECT
            actor_id
        FROM
            community c
        WHERE
            cf.community_id = c.id) AS community_actor_id,
    (
        SELECT
            local
        FROM
            community c
        WHERE
            cf.community_id = c.id) AS community_local,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            cf.community_id = c.id) AS community_name
FROM
    community_follower cf;

CREATE OR REPLACE VIEW community_user_ban_view AS
SELECT
    *,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            cm.user_id = u.id), (
        SELECT
            actor_id
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_actor_id,
    (
        SELECT
            local
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_local,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_name
FROM
    community_user_ban cm;

