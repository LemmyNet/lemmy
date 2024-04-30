ALTER TABLE custom_emoji
    ADD COLUMN local_site_id int REFERENCES local_site (site_id) ON UPDATE CASCADE ON DELETE CASCADE;

UPDATE
    custom_emoji
SET
    local_site_id = (
        SELECT
            site_id
        FROM
            local_site
        LIMIT 1);

ALTER TABLE custom_emoji
    ALTER COLUMN local_site_id SET NOT NULL;

ALTER TABLE tagline
    ADD COLUMN local_site_id int REFERENCES local_site (site_id) ON UPDATE CASCADE ON DELETE CASCADE;

UPDATE
    tagline
SET
    local_site_id = (
        SELECT
            site_id
        FROM
            local_site
        LIMIT 1);

ALTER TABLE tagline
    ALTER COLUMN local_site_id SET NOT NULL;

