DROP TABLE activity;

DROP VIEW community_view, community_mview;

DROP MATERIALIZED VIEW community_aggregates_mview;

DROP VIEW community_aggregates_view;

ALTER TABLE user_
    DROP COLUMN actor_id,
    DROP COLUMN private_key,
    DROP COLUMN public_key,
    DROP COLUMN bio,
    DROP COLUMN local,
    DROP COLUMN last_refreshed_at;

ALTER TABLE community
    DROP COLUMN actor_id,
    DROP COLUMN private_key,
    DROP COLUMN public_key,
    DROP COLUMN local,
    DROP COLUMN last_refreshed_at;

-- Views are the same as before, except `*` does not reference the dropped columns
CREATE VIEW community_aggregates_view AS
SELECT
    c.*,
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

