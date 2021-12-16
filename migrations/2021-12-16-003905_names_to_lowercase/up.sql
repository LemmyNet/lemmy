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
alter table person add column old_name varchar(255);
update person set old_name = name;

alter table person add column old_actor_id varchar(255);
update person set old_actor_id = actor_id;

alter table person add column old_inbox_url varchar(255);
update person set old_inbox_url = inbox_url;


-- Set person names, actor_id, and inbox_url to lowercase
update person set name = lower(name);
update person set actor_id = lower(actor_id);
update person set inbox_url = lower(inbox_url);



-- Add a lowecase enforcement check to these three columns

alter table person add constraint person_name_lowercase_ck check (name = lower(name));
alter table person add constraint person_actor_id_lowercase_ck check (actor_id = lower(actor_id));
alter table person add constraint person_inbox_url_lowercase_ck check (inbox_url = lower(inbox_url));

