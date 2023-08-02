-- Drop the columns
DROP VIEW site_view;

ALTER TABLE site
    DROP COLUMN enable_downvotes;

ALTER TABLE site
    DROP COLUMN open_registration;

ALTER TABLE site
    DROP COLUMN enable_nsfw;

-- Rebuild the views
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

