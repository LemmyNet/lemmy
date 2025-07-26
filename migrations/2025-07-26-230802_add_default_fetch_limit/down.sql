-- Drop the new columns
ALTER TABLE local_user
    DROP COLUMN default_fetch_limit;

ALTER TABLE local_site
    DROP COLUMN default_fetch_limit;

