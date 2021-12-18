-- This file should undo anything in `up.sql`



-- Remove column constraints
alter table person drop constraint idx_person_name_lowercase;
alter table person drop constraint idx_person_actor_id_lowercase;
alter table person drop constraint idx_person_inbox_url_lowercase;


-- Recreate the unique index idx_person_lower_actor_id 
create unique index idx_person_lower_actor_id on person (lower(actor_id));
