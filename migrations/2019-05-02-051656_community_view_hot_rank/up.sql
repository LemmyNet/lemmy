DROP VIEW community_view;

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

