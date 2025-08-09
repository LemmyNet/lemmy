-- Drop the new columns
ALTER TABLE local_user
    DROP COLUMN default_items_per_page;

ALTER TABLE local_site
    DROP COLUMN default_items_per_page;

