-- Your SQL goes here

-- Delete actor_id lowercase dupes, keeping the first one

delete from person a
    using person b
    where a.id > b.id
    and lower(a.actor_id) = lower(b.actor_id);



-- Set person names, actor_id, and inbox_url to lowercase
update person set name = lower(name);
update person set actor_id = lower(actor_id);
update person set inbox_url = lower(inbox_url);


-- Add a lowecase enforcement check to these three columns

alter table person add constraint idx_person_name_lowercase check (name = lower(name));
alter table person add constraint idx_person_actor_id_lowercase check (actor_id = lower(actor_id));
alter table person add constraint idx_person_inbox_url_lowercase check (inbox_url = lower(inbox_url));

-- Remove the previous unique index for lower_actor_id
drop index idx_person_lower_actor_id;
