-- Some fixes
ALTER TABLE community
    ALTER COLUMN hidden DROP NOT NULL;

ALTER TABLE community
    ALTER COLUMN posting_restricted_to_mods DROP NOT NULL;

ALTER TABLE activity
    ALTER COLUMN sensitive DROP NOT NULL;

ALTER TABLE mod_add
    ALTER COLUMN removed DROP NOT NULL;

ALTER TABLE mod_add_community
    ALTER COLUMN removed DROP NOT NULL;

ALTER TABLE mod_ban
    ALTER COLUMN banned DROP NOT NULL;

ALTER TABLE mod_ban_from_community
    ALTER COLUMN banned DROP NOT NULL;

ALTER TABLE mod_hide_community
    ALTER COLUMN hidden DROP NOT NULL;

ALTER TABLE mod_lock_post
    ALTER COLUMN LOCKED DROP NOT NULL;

ALTER TABLE mod_remove_comment
    ALTER COLUMN removed DROP NOT NULL;

ALTER TABLE mod_remove_community
    ALTER COLUMN removed DROP NOT NULL;

ALTER TABLE mod_remove_post
    ALTER COLUMN removed DROP NOT NULL;

ALTER TABLE mod_transfer_community
    ADD COLUMN removed boolean DEFAULT FALSE;

ALTER TABLE LANGUAGE
    ALTER COLUMN code DROP NOT NULL;

ALTER TABLE LANGUAGE
    ALTER COLUMN name DROP NOT NULL;

-- Fix the registration mode enums
ALTER TYPE registration_mode_enum RENAME VALUE 'Closed' TO 'closed';

ALTER TYPE registration_mode_enum RENAME VALUE 'RequireApplication' TO 'require_application';

ALTER TYPE registration_mode_enum RENAME VALUE 'Open' TO 'open';

-- add back old columns
-- Alter the local_user table
ALTER TABLE local_user
    ALTER COLUMN default_sort_type DROP DEFAULT;

ALTER TABLE local_user
    ALTER COLUMN default_sort_type TYPE smallint
    USING
        CASE default_sort_type
        WHEN 'Active' THEN
            0
        WHEN 'Hot' THEN
            1
        WHEN 'New' THEN
            2
        WHEN 'Old' THEN
            3
        WHEN 'TopDay' THEN
            4
        WHEN 'TopWeek' THEN
            5
        WHEN 'TopMonth' THEN
            6
        WHEN 'TopYear' THEN
            7
        WHEN 'TopAll' THEN
            8
        WHEN 'MostComments' THEN
            9
        WHEN 'NewComments' THEN
            10
        ELSE
            0
        END;

ALTER TABLE local_user
    ALTER COLUMN default_sort_type SET DEFAULT 0;

ALTER TABLE local_user
    ALTER COLUMN default_listing_type DROP DEFAULT;

ALTER TABLE local_user
    ALTER COLUMN default_listing_type TYPE smallint
    USING
        CASE default_listing_type
        WHEN 'All' THEN
            0
        WHEN 'Local' THEN
            1
        WHEN 'Subscribed' THEN
            2
        ELSE
            1
        END;

ALTER TABLE local_user
    ALTER COLUMN default_listing_type SET DEFAULT 1;

-- Alter the local site column
ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type DROP DEFAULT;

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type TYPE text;

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type SET DEFAULT 'Local';

-- Drop the types
DROP TYPE listing_type_enum;

DROP TYPE sort_type_enum;

