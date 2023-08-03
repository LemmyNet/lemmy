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
                AND p.id = co.post_id) AS number_of_comments
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

CREATE VIEW community_moderator_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_name
FROM
    community_moderator cm;

CREATE VIEW community_follower_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_name,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            cf.community_id = c.id) AS community_name
FROM
    community_follower cf;

CREATE VIEW community_user_ban_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            name
        FROM
            community c
        WHERE
            cm.community_id = c.id) AS community_name
FROM
    community_user_ban cm;

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
            comment) AS number_of_comments
FROM
    site s;

