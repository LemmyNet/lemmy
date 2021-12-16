-- This file should undo anything in `up.sql`


-- Recover the old values
update person set name = old_name;
update person set actor_id = old_actor_id;
update person set inbox_url = old_inbox_url;

-- Delete the columns with the "old" values
alter table person drop column old_name;
alter table person drop column old_actor_id;
alter table person drop column old_inbox_url;

-- Remove column constraints
alter table person drop constraint person_inbox_url_lowercase_ck;
alter table person drop constraint person_actor_id_lowercase_ck;
alter table person drop constraint person_inbox_url_lowercase_ck;
