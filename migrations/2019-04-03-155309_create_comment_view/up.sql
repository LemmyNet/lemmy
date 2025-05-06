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

