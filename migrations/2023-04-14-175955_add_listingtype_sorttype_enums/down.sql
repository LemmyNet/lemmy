-- Some fixes
alter table community alter column hidden drop not null;
alter table community alter column posting_restricted_to_mods drop not null;
alter table activity alter column sensitive drop not null;
alter table mod_add alter column removed drop not null;
alter table mod_add_community alter column removed drop not null;
alter table mod_ban alter column banned drop not null;
alter table mod_ban_from_community alter column banned drop not null;
alter table mod_hide_community alter column hidden drop not null;
alter table mod_lock_post alter column locked drop not null;
alter table mod_remove_comment alter column removed drop not null;
alter table mod_remove_community alter column removed drop not null;
alter table mod_remove_post alter column removed drop not null;
alter table mod_transfer_community add column removed boolean default false;
alter table language alter column code drop not null;
alter table language alter column name drop not null;

-- Fix the registration mode enums
ALTER TYPE registration_mode_enum RENAME VALUE 'Closed' TO 'closed';
ALTER TYPE registration_mode_enum RENAME VALUE 'RequireApplication' TO 'require_application';
ALTER TYPE registration_mode_enum RENAME VALUE 'Open' TO 'open';

-- add back old columns

-- Alter the local_user table
alter table local_user alter column default_sort_type drop default;
alter table local_user alter column default_sort_type type smallint using
    case default_sort_type
        when 'Active' then 0
        when 'Hot' then 1
        when 'New' then 2
        when 'Old' then 3
        when 'TopDay' then 4
        when 'TopWeek' then 5
        when 'TopMonth' then 6
        when 'TopYear' then 7
        when 'TopAll' then 8
        when 'MostComments' then 9
        when 'NewComments' then 10
        else 0
    end;
alter table local_user alter column default_sort_type set default 0;

alter table local_user alter column default_listing_type drop default;
alter table local_user alter column default_listing_type type smallint using
    case default_listing_type
        when 'All' then 0
        when 'Local' then 1
        when 'Subscribed' then 2
        else 1
    end;
alter table local_user alter column default_listing_type set default 1;

-- Alter the local site column

alter table local_site alter column default_post_listing_type drop default;
alter table local_site alter column default_post_listing_type type text;
alter table local_site alter column default_post_listing_type set default 1;

-- Drop the types
drop type listing_type_enum;
drop type sort_type_enum;
