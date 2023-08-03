ALTER TABLE user_
    ADD COLUMN default_sort_type smallint DEFAULT 0 NOT NULL;

ALTER TABLE user_
    ADD COLUMN default_listing_type smallint DEFAULT 1 NOT NULL;

