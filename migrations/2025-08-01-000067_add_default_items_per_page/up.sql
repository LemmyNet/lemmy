-- Adds an optional default fetch limit (IE fetch a certain number of posts) to local_user and local_site
ALTER TABLE local_user
    ADD COLUMN default_items_per_page integer NOT NULL DEFAULT 20;

ALTER TABLE local_site
    ADD COLUMN default_items_per_page integer NOT NULL DEFAULT 20;

