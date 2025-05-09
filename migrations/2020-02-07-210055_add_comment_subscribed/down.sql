DROP VIEW reply_view;

DROP VIEW user_mention_view;

DROP VIEW user_mention_mview;

DROP VIEW comment_view;

DROP VIEW comment_mview;

DROP MATERIALIZED VIEW comment_aggregates_mview;

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
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
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
    um.recipient_id
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
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS saved,
    um.recipient_id
FROM
    all_comment ac
    LEFT JOIN user_mention um ON um.comment_id = ac.id;

