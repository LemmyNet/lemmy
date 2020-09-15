-- Following this issue : https://github.com/LemmyNet/lemmy/issues/957

-- Creating a unique changeme actor_id
create or replace function generate_unique_changeme() 
returns text language sql 
as $$
  select 'changeme_' || string_agg (substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil (random() * 62)::integer, 1), '')
  from generate_series(1, 20)
$$;

-- Need to delete the possible community and user dupes for ones that don't start with the fake one
-- A few test inserts, to make sure this removes later dupes
-- insert into community (name, title, category_id, creator_id) values ('testcom', 'another testcom', 1, 2);
delete from community a using (
  select min(id) as id, actor_id
    from community 
    group by actor_id having count(*) > 1
) b
where a.actor_id = b.actor_id 
and a.id <> b.id;

delete from user_ a using (
  select min(id) as id, actor_id
    from user_ 
    group by actor_id having count(*) > 1
) b
where a.actor_id = b.actor_id 
and a.id <> b.id;

-- Replacing the current default on the columns, to the unique one
update community 
set actor_id = generate_unique_changeme()
where actor_id = 'http://fake.com';

update user_ 
set actor_id = generate_unique_changeme()
where actor_id = 'http://fake.com';

-- Add the unique indexes
alter table community alter column actor_id set not null;
alter table community alter column actor_id set default generate_unique_changeme();

alter table user_ alter column actor_id set not null;
alter table user_ alter column actor_id set default generate_unique_changeme();

-- Add lowercase uniqueness too
drop index idx_user_name_lower_actor_id;
create unique index idx_user_lower_actor_id on user_ (lower(actor_id));

create unique index idx_community_lower_actor_id on community (lower(actor_id));
