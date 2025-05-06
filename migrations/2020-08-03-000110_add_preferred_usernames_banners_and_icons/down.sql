-- Drops first
DROP VIEW site_view;

DROP TABLE user_fast;

DROP VIEW user_view;

DROP VIEW post_fast_view;

DROP TABLE post_aggregates_fast;

DROP VIEW post_view;

DROP VIEW post_aggregates_view;

DROP VIEW community_moderator_view;

DROP VIEW community_follower_view;

DROP VIEW community_user_ban_view;

DROP VIEW community_view;

DROP VIEW community_aggregates_view;

DROP VIEW community_fast_view;

DROP TABLE community_aggregates_fast;

DROP VIEW private_message_view;

DROP VIEW user_mention_view;

DROP VIEW reply_fast_view;

DROP VIEW comment_fast_view;

DROP VIEW comment_view;

DROP VIEW user_mention_fast_view;

DROP TABLE comment_aggregates_fast;

DROP VIEW comment_aggregates_view;

ALTER TABLE site
    DROP COLUMN icon,
    DROP COLUMN banner;

ALTER TABLE community
    DROP COLUMN icon,
    DROP COLUMN banner;

ALTER TABLE user_
    DROP COLUMN banner;

-- Site
CREATE VIEW site_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            s.creator_id = u.id) AS creator_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            s.creator_id = u.id) AS creator_avatar,
    (
        SELECT
            count(*)
        FROM
            user_) AS number_of_users,
    (
        SELECT
            count(*)
        FROM
            post) AS number_of_posts,
    (
        SELECT
            count(*)
        FROM
            comment) AS number_of_comments,
    (
        SELECT
            count(*)
        FROM
            community) AS number_of_communities
FROM
    site s;

-- User
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
    coalesce(pd.posts, 0) AS number_of_posts,
    coalesce(pd.score, 0) AS post_score,
    coalesce(cd.comments, 0) AS number_of_comments,
    coalesce(cd.score, 0) AS comment_score
FROM
    user_ u
    LEFT JOIN (
        SELECT
            p.creator_id AS creator_id,
            count(DISTINCT p.id) AS posts,
            sum(pl.score) AS score
        FROM
            post p
            JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            p.creator_id) pd ON u.id = pd.creator_id
    LEFT JOIN (
        SELECT
            c.creator_id,
            count(DISTINCT c.id) AS comments,
            sum(cl.score) AS score
        FROM
            comment c
            JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            c.creator_id) cd ON u.id = cd.creator_id;

CREATE TABLE user_fast AS
SELECT
    *
FROM
    user_view;

ALTER TABLE user_fast
    ADD PRIMARY KEY (id);

-- Post fast
CREATE VIEW post_aggregates_view AS
SELECT
    p.*,
    -- creator details
    u.actor_id AS creator_actor_id,
    u."local" AS creator_local,
    u."name" AS creator_name,
    u.published AS creator_published,
    u.avatar AS creator_avatar,
    u.banned AS banned,
    cb.id::bool AS banned_from_community,
    -- community details
    c.actor_id AS community_actor_id,
    c."local" AS community_local,
    c."name" AS community_name,
    c.removed AS community_removed,
    c.deleted AS community_deleted,
    c.nsfw AS community_nsfw,
    -- post score data/comment count
    coalesce(ct.comments, 0) AS number_of_comments,
    coalesce(pl.score, 0) AS score,
    coalesce(pl.upvotes, 0) AS upvotes,
    coalesce(pl.downvotes, 0) AS downvotes,
    hot_rank (coalesce(pl.score, 0), (
            CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
                p.published
            ELSE
                greatest (ct.recent_comment_time, p.published)
            END)) AS hot_rank,
    (
        CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
            p.published
        ELSE
            greatest (ct.recent_comment_time, p.published)
        END) AS newest_activity_time
FROM
    post p
    LEFT JOIN user_ u ON p.creator_id = u.id
    LEFT JOIN community_user_ban cb ON p.creator_id = cb.user_id
        AND p.community_id = cb.community_id
    LEFT JOIN community c ON p.community_id = c.id
    LEFT JOIN (
        SELECT
            post_id,
            count(*) AS comments,
            max(published) AS recent_comment_time
        FROM
            comment
        GROUP BY
            post_id) ct ON ct.post_id = p.id
    LEFT JOIN (
        SELECT
            post_id,
            sum(score) AS score,
            sum(score) FILTER (WHERE score = 1) AS upvotes,
            - sum(score) FILTER (WHERE score = - 1) AS downvotes
        FROM
            post_like
        GROUP BY
            post_id) pl ON pl.post_id = p.id
ORDER BY
    p.id;

CREATE VIEW post_view AS
SELECT
    pav.*,
    us.id AS user_id,
    us.user_vote AS my_vote,
    us.is_subbed::bool AS subscribed,
    us.is_read::bool AS read,
    us.is_saved::bool AS saved
FROM
    post_aggregates_view pav
    CROSS JOIN LATERAL (
        SELECT
            u.id,
            coalesce(cf.community_id, 0) AS is_subbed,
            coalesce(pr.post_id, 0) AS is_read,
            coalesce(ps.post_id, 0) AS is_saved,
            coalesce(pl.score, 0) AS user_vote
        FROM
            user_ u
            LEFT JOIN community_user_ban cb ON u.id = cb.user_id
                AND cb.community_id = pav.community_id
        LEFT JOIN community_follower cf ON u.id = cf.user_id
            AND cf.community_id = pav.community_id
    LEFT JOIN post_read pr ON u.id = pr.user_id
        AND pr.post_id = pav.id
    LEFT JOIN post_saved ps ON u.id = ps.user_id
        AND ps.post_id = pav.id
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND pav.id = pl.post_id) AS us
UNION ALL
SELECT
    pav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS read,
    NULL AS saved
FROM
    post_aggregates_view pav;

CREATE TABLE post_aggregates_fast AS
SELECT
    *
FROM
    post_aggregates_view;

ALTER TABLE post_aggregates_fast
    ADD PRIMARY KEY (id);

CREATE VIEW post_fast_view AS
SELECT
    pav.*,
    us.id AS user_id,
    us.user_vote AS my_vote,
    us.is_subbed::bool AS subscribed,
    us.is_read::bool AS read,
    us.is_saved::bool AS saved
FROM
    post_aggregates_fast pav
    CROSS JOIN LATERAL (
        SELECT
            u.id,
            coalesce(cf.community_id, 0) AS is_subbed,
            coalesce(pr.post_id, 0) AS is_read,
            coalesce(ps.post_id, 0) AS is_saved,
            coalesce(pl.score, 0) AS user_vote
        FROM
            user_ u
            LEFT JOIN community_user_ban cb ON u.id = cb.user_id
                AND cb.community_id = pav.community_id
        LEFT JOIN community_follower cf ON u.id = cf.user_id
            AND cf.community_id = pav.community_id
    LEFT JOIN post_read pr ON u.id = pr.user_id
        AND pr.post_id = pav.id
    LEFT JOIN post_saved ps ON u.id = ps.user_id
        AND ps.post_id = pav.id
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND pav.id = pl.post_id) AS us
UNION ALL
SELECT
    pav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS read,
    NULL AS saved
FROM
    post_aggregates_fast pav;

-- Community
CREATE VIEW community_aggregates_view AS
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
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.avatar AS creator_avatar,
    cat.name AS category_name,
    coalesce(cf.subs, 0) AS number_of_subscribers,
    coalesce(cd.posts, 0) AS number_of_posts,
    coalesce(cd.comments, 0) AS number_of_comments,
    hot_rank (cf.subs, c.published) AS hot_rank
FROM
    community c
    LEFT JOIN user_ u ON c.creator_id = u.id
    LEFT JOIN category cat ON c.category_id = cat.id
    LEFT JOIN (
        SELECT
            p.community_id,
            count(DISTINCT p.id) AS posts,
            count(DISTINCT ct.id) AS comments
        FROM
            post p
            JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            p.community_id) cd ON cd.community_id = c.id
    LEFT JOIN (
        SELECT
            community_id,
            count(*) AS subs
        FROM
            community_follower
        GROUP BY
            community_id) cf ON cf.community_id = c.id;

CREATE VIEW community_view AS
SELECT
    cv.*,
    us.user AS user_id,
    us.is_subbed::bool AS subscribed
FROM
    community_aggregates_view cv
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user,
            coalesce(cf.community_id, 0) AS is_subbed
        FROM
            user_ u
            LEFT JOIN community_follower cf ON u.id = cf.user_id
                AND cf.community_id = cv.id) AS us
UNION ALL
SELECT
    cv.*,
    NULL AS user_id,
    NULL AS subscribed
FROM
    community_aggregates_view cv;

CREATE VIEW community_moderator_view AS
SELECT
    cm.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name
FROM
    community_moderator cm
    LEFT JOIN user_ u ON cm.user_id = u.id
    LEFT JOIN community c ON cm.community_id = c.id;

CREATE VIEW community_follower_view AS
SELECT
    cf.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name
FROM
    community_follower cf
    LEFT JOIN user_ u ON cf.user_id = u.id
    LEFT JOIN community c ON cf.community_id = c.id;

CREATE VIEW community_user_ban_view AS
SELECT
    cb.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS community_actor_id,
    c.local AS community_local,
    c.name AS community_name
FROM
    community_user_ban cb
    LEFT JOIN user_ u ON cb.user_id = u.id
    LEFT JOIN community c ON cb.community_id = c.id;

-- The community fast table
CREATE TABLE community_aggregates_fast AS
SELECT
    *
FROM
    community_aggregates_view;

ALTER TABLE community_aggregates_fast
    ADD PRIMARY KEY (id);

CREATE VIEW community_fast_view AS
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
    CROSS JOIN (
        SELECT
            ca.*
        FROM
            community_aggregates_fast ca) ac
UNION ALL
SELECT
    caf.*,
    NULL AS user_id,
    NULL AS subscribed
FROM
    community_aggregates_fast caf;

-- Private message
CREATE VIEW private_message_view AS
SELECT
    pm.*,
    u.name AS creator_name,
    u.avatar AS creator_avatar,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u2.name AS recipient_name,
    u2.avatar AS recipient_avatar,
    u2.actor_id AS recipient_actor_id,
    u2.local AS recipient_local
FROM
    private_message pm
    INNER JOIN user_ u ON u.id = pm.creator_id
    INNER JOIN user_ u2 ON u2.id = pm.recipient_id;

-- Comments, mentions, replies
CREATE VIEW comment_aggregates_view AS
SELECT
    ct.*,
    -- post details
    p."name" AS post_name,
    p.community_id,
    -- community details
    c.actor_id AS community_actor_id,
    c."local" AS community_local,
    c."name" AS community_name,
    -- creator details
    u.banned AS banned,
    coalesce(cb.id, 0)::bool AS banned_from_community,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.published AS creator_published,
    u.avatar AS creator_avatar,
    -- score details
    coalesce(cl.total, 0) AS score,
    coalesce(cl.up, 0) AS upvotes,
    coalesce(cl.down, 0) AS downvotes,
    hot_rank (coalesce(cl.total, 0), ct.published) AS hot_rank
FROM
    comment ct
    LEFT JOIN post p ON ct.post_id = p.id
    LEFT JOIN community c ON p.community_id = c.id
    LEFT JOIN user_ u ON ct.creator_id = u.id
    LEFT JOIN community_user_ban cb ON ct.creator_id = cb.user_id
        AND p.id = ct.post_id
        AND p.community_id = cb.community_id
    LEFT JOIN (
        SELECT
            l.comment_id AS id,
            sum(l.score) AS total,
            count(
                CASE WHEN l.score = 1 THEN
                    1
                ELSE
                    NULL
                END) AS up,
            count(
                CASE WHEN l.score = - 1 THEN
                    1
                ELSE
                    NULL
                END) AS down
        FROM
            comment_like l
        GROUP BY
            comment_id) AS cl ON cl.id = ct.id;

CREATE OR REPLACE VIEW comment_view AS (
    SELECT
        cav.*,
        us.user_id AS user_id,
        us.my_vote AS my_vote,
        us.is_subbed::bool AS subscribed,
        us.is_saved::bool AS saved
    FROM
        comment_aggregates_view cav
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user_id,
            coalesce(cl.score, 0) AS my_vote,
            coalesce(cf.id, 0) AS is_subbed,
            coalesce(cs.id, 0) AS is_saved
        FROM
            user_ u
            LEFT JOIN comment_like cl ON u.id = cl.user_id
                AND cav.id = cl.comment_id
        LEFT JOIN comment_saved cs ON u.id = cs.user_id
            AND cs.comment_id = cav.id
    LEFT JOIN community_follower cf ON u.id = cf.user_id
        AND cav.community_id = cf.community_id) AS us
UNION ALL
SELECT
    cav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS saved
FROM
    comment_aggregates_view cav);

CREATE TABLE comment_aggregates_fast AS
SELECT
    *
FROM
    comment_aggregates_view;

ALTER TABLE comment_aggregates_fast
    ADD PRIMARY KEY (id);

CREATE VIEW comment_fast_view AS
SELECT
    cav.*,
    us.user_id AS user_id,
    us.my_vote AS my_vote,
    us.is_subbed::bool AS subscribed,
    us.is_saved::bool AS saved
FROM
    comment_aggregates_fast cav
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user_id,
            coalesce(cl.score, 0) AS my_vote,
            coalesce(cf.id, 0) AS is_subbed,
            coalesce(cs.id, 0) AS is_saved
        FROM
            user_ u
            LEFT JOIN comment_like cl ON u.id = cl.user_id
                AND cav.id = cl.comment_id
        LEFT JOIN comment_saved cs ON u.id = cs.user_id
            AND cs.comment_id = cav.id
    LEFT JOIN community_follower cf ON u.id = cf.user_id
        AND cav.community_id = cf.community_id) AS us
UNION ALL
SELECT
    cav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS saved
FROM
    comment_aggregates_fast cav;

CREATE VIEW user_mention_view AS
SELECT
    c.id,
    um.id AS user_mention_id,
    c.creator_id,
    c.creator_actor_id,
    c.creator_local,
    c.post_id,
    c.post_name,
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

CREATE VIEW user_mention_fast_view AS
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.post_name,
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
    CROSS JOIN (
        SELECT
            ca.*
        FROM
            comment_aggregates_fast ca) ac
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
    ac.post_name,
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
    comment_aggregates_fast ac
    LEFT JOIN user_mention um ON um.comment_id = ac.id;

-- Do the reply_view referencing the comment_fast_view
CREATE VIEW reply_fast_view AS
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
    comment_fast_view cv,
    closereply
WHERE
    closereply.id = cv.id;

-- redoing the triggers
CREATE OR REPLACE FUNCTION refresh_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        -- Update community number of posts
        UPDATE
            community_aggregates_fast
        SET
            number_of_posts = number_of_posts - 1
        WHERE
            id = OLD.community_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
        -- Update that users number of posts, post score
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id;
        -- Update community number of posts
        UPDATE
            community_aggregates_fast
        SET
            number_of_posts = number_of_posts + 1
        WHERE
            id = NEW.community_id;
        -- Update the hot rank on the post table
        -- TODO this might not correctly update it, using a 1 week interval
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = pav.hot_rank
        FROM
            post_aggregates_view AS pav
        WHERE
            paf.id = pav.id
            AND (pav.published > ('now'::timestamp - '1 week'::interval));
    END IF;
    RETURN NULL;
END
$$;

CREATE OR REPLACE FUNCTION refresh_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        -- Update community number of comments
        UPDATE
            community_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments - 1
        FROM
            post AS p
        WHERE
            caf.id = p.community_id
            AND p.id = OLD.post_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
        -- Update user view due to comment count
        UPDATE
            user_fast
        SET
            number_of_comments = number_of_comments + 1
        WHERE
            id = NEW.creator_id;
        -- Update post view due to comment count, new comment activity time, but only on new posts
        -- TODO this could be done more efficiently
        DELETE FROM post_aggregates_fast
        WHERE id = NEW.post_id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.post_id;
        -- Force the hot rank as zero on week-older posts
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = 0
        WHERE
            paf.id = NEW.post_id
            AND (paf.published < ('now'::timestamp - '1 week'::interval));
        -- Update community number of comments
        UPDATE
            community_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments + 1
        FROM
            post AS p
        WHERE
            caf.id = p.community_id
            AND p.id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

