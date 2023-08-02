-- Add the column
ALTER TABLE site
    ADD COLUMN enable_downvotes boolean DEFAULT TRUE NOT NULL;

ALTER TABLE site
    ADD COLUMN open_registration boolean DEFAULT TRUE NOT NULL;

ALTER TABLE site
    ADD COLUMN enable_nsfw boolean DEFAULT TRUE NOT NULL;

-- Reload the view
DROP VIEW site_view;

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
            comment) AS number_of_comments,
    (
        SELECT
            count(*)
        FROM
            community) AS number_of_communities
FROM
    site s;

