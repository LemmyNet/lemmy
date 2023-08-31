DROP VIEW user_mention_view;

DROP VIEW reply_fast_view;

DROP VIEW comment_fast_view;

DROP VIEW comment_view;

DROP VIEW user_mention_fast_view;

DROP TABLE comment_aggregates_fast;

DROP VIEW comment_aggregates_view;

CREATE VIEW comment_aggregates_view AS
SELECT
    ct.*,
    -- community details
    p.community_id,
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

