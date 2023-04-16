-- A few DB fixes
alter table community alter column hidden set not null;
alter table community alter column posting_restricted_to_mods set not null;
alter table activity alter column sensitive set not null;
alter table mod_add alter column removed set not null;
alter table mod_add_community alter column removed set not null;
alter table mod_ban alter column banned set not null;
alter table mod_ban_from_community alter column banned set not null;
alter table mod_hide_community alter column hidden set not null;
alter table mod_lock_post alter column locked set not null;
alter table mod_remove_comment alter column removed set not null;
alter table mod_remove_community alter column removed set not null;
alter table mod_remove_post alter column removed set not null;
alter table mod_transfer_community drop column removed;
alter table language alter column code set not null;
alter table language alter column name set not null;

-- Fix the registration mode enums
ALTER TYPE registration_mode_enum RENAME VALUE 'closed' TO 'Closed';
ALTER TYPE registration_mode_enum RENAME VALUE 'require_application' TO 'RequireApplication';
ALTER TYPE registration_mode_enum RENAME VALUE 'open' TO 'Open';

-- Create the enums

CREATE TYPE sort_type_enum AS ENUM ('Active', 'Hot', 'New', 'Old', 'TopDay', 'TopWeek', 'TopMonth', 'TopYear', 'TopAll', 'MostComments', 'NewComments');
  
CREATE TYPE listing_type_enum AS ENUM ('All', 'Local', 'Subscribed');

-- Alter the local_user table
alter table local_user alter column default_sort_type drop default;
alter table local_user alter column default_sort_type type sort_type_enum using
    case default_sort_type
        when 0 then 'Active'
        when 1 then 'Hot'
        when 2 then 'New'
        when 3 then 'Old'
        when 4 then 'TopDay'
        when 5 then 'TopWeek'
        when 6 then 'TopMonth'
        when 7 then 'TopYear'
        when 8 then 'TopAll'
        when 9 then 'MostComments'
        when 10 then 'NewComments'
        else 'Active'
    end :: sort_type_enum;
alter table local_user alter column default_sort_type set default 'Active';

alter table local_user alter column default_listing_type drop default;
alter table local_user alter column default_listing_type type listing_type_enum using
    case default_listing_type
        when 0 then 'All'
        when 1 then 'Local'
        when 2 then 'Subscribed'
        else 'Local'
    end :: listing_type_enum;
alter table local_user alter column default_listing_type set default 'Local';

-- Alter the local site column
alter table local_site alter column default_post_listing_type drop default;
alter table local_site alter column default_post_listing_type type listing_type_enum using default_post_listing_type::listing_type_enum;
alter table local_site alter column default_post_listing_type set default 'Local';
