-- Add the bot_account column to the person table
drop view person_alias_1, person_alias_2;
alter table person add column bot_account boolean not null default false;
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;

-- Add the show_bot_accounts to the local user table as a setting
alter table local_user add column show_bot_accounts boolean not null default true;
