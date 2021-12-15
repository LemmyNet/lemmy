-- Your SQL goes here


-- Delete person name dupes, keeping the first one

delete
from person
where id not in (
  select min(id)
  from person
  group by lower(actor_id)
);

-- Store the old values so that down.sql can recover them
create unique index old_name on person (name);
create unique index old_actor_id on person (actor_id);
create unique index old_inbox_url on person (inbox_url);



-- Set person names, actor_id, and inbox_url to lowercase

update person set name = lower(name);
update person set actor_id = lower(actor_id);
update person set inbox_url = lower(inbox_url);



-- Add a lowecase enforcement check to these three columns

alter table person add constraint person_name_lowercase_ck check (name = lower(name));
alter table person add constraint person_actor_id_lowercase_ck check (actor_id = lower(actor_id));
alter table person add constraint person_inbox_url_lowercase_ck check (inbox_url = lower(inbox_url));

