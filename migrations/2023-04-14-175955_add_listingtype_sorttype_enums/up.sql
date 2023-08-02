-- A few DB fixes
ALTER TABLE community
    ALTER COLUMN hidden SET NOT NULL;

ALTER TABLE community
    ALTER COLUMN posting_restricted_to_mods SET NOT NULL;

ALTER TABLE activity
    ALTER COLUMN sensitive SET NOT NULL;

ALTER TABLE mod_add
    ALTER COLUMN removed SET NOT NULL;

ALTER TABLE mod_add_community
    ALTER COLUMN removed SET NOT NULL;

ALTER TABLE mod_ban
    ALTER COLUMN banned SET NOT NULL;

ALTER TABLE mod_ban_from_community
    ALTER COLUMN banned SET NOT NULL;

ALTER TABLE mod_hide_community
    ALTER COLUMN hidden SET NOT NULL;

ALTER TABLE mod_lock_post
    ALTER COLUMN LOCKED SET NOT NULL;

ALTER TABLE mod_remove_comment
    ALTER COLUMN removed SET NOT NULL;

ALTER TABLE mod_remove_community
    ALTER COLUMN removed SET NOT NULL;

ALTER TABLE mod_remove_post
    ALTER COLUMN removed SET NOT NULL;

ALTER TABLE mod_transfer_community
    DROP COLUMN removed;

ALTER TABLE LANGUAGE
    ALTER COLUMN code SET NOT NULL;

ALTER TABLE LANGUAGE
    ALTER COLUMN name SET NOT NULL;

-- Fix the registration mode enums
ALTER TYPE registration_mode_enum RENAME VALUE 'closed' TO 'Closed';

ALTER TYPE registration_mode_enum RENAME VALUE 'require_application' TO 'RequireApplication';

ALTER TYPE registration_mode_enum RENAME VALUE 'open' TO 'Open';

-- Create the enums
CREATE TYPE sort_type_enum AS ENUM (
    'Active',
    'Hot',
    'New',
    'Old',
    'TopDay',
    'TopWeek',
    'TopMonth',
    'TopYear',
    'TopAll',
    'MostComments',
    'NewComments'
);

CREATE TYPE listing_type_enum AS ENUM (
    'All',
    'Local',
    'Subscribed'
);

-- Alter the local_user table
ALTER TABLE local_user
    ALTER COLUMN default_sort_type DROP DEFAULT;

ALTER TABLE local_user
    ALTER COLUMN default_sort_type TYPE sort_type_enum
    USING
        CASE default_sort_type
        WHEN 0 THEN
            'Active'
        WHEN 1 THEN
            'Hot'
        WHEN 2 THEN
            'New'
        WHEN 3 THEN
            'Old'
        WHEN 4 THEN
            'TopDay'
        WHEN 5 THEN
            'TopWeek'
        WHEN 6 THEN
            'TopMonth'
        WHEN 7 THEN
            'TopYear'
        WHEN 8 THEN
            'TopAll'
        WHEN 9 THEN
            'MostComments'
        WHEN 10 THEN
            'NewComments'
        ELSE
            'Active'
        END::sort_type_enum;

ALTER TABLE local_user
    ALTER COLUMN default_sort_type SET DEFAULT 'Active';

ALTER TABLE local_user
    ALTER COLUMN default_listing_type DROP DEFAULT;

ALTER TABLE local_user
    ALTER COLUMN default_listing_type TYPE listing_type_enum
    USING
        CASE default_listing_type
        WHEN 0 THEN
            'All'
        WHEN 1 THEN
            'Local'
        WHEN 2 THEN
            'Subscribed'
        ELSE
            'Local'
        END::listing_type_enum;

ALTER TABLE local_user
    ALTER COLUMN default_listing_type SET DEFAULT 'Local';

-- Alter the local site column
ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type DROP DEFAULT;

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type TYPE listing_type_enum
    USING default_post_listing_type::listing_type_enum;

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type SET DEFAULT 'Local';

