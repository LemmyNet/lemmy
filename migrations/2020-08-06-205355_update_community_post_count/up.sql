-- Drop first
DROP VIEW community_view;

DROP VIEW community_aggregates_view;

DROP VIEW community_fast_view;

DROP TABLE community_aggregates_fast;

CREATE VIEW community_aggregates_view AS
SELECT
    c.id,
    c.name,
    c.title,
    c.icon,
    c.banner,
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
    u.preferred_username AS creator_preferred_username,
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
            LEFT JOIN comment ct ON p.id = ct.post_id
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

