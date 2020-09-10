
alter table community alter column actor_id set not null;
alter table community alter column actor_id set default 'http://fake.com';
alter table user_ alter column actor_id set not null;
alter table user_ alter column actor_id set default 'http://fake.com';

drop function generate_unique_changeme;

update community
set actor_id = 'http://fake.com'
where actor_id like 'changeme_%';

update user_
set actor_id = 'http://fake.com'
where actor_id like 'changeme_%';

drop index idx_user_lower_actor_id;
create unique index idx_user_name_lower_actor_id on user_ (lower(name), lower(actor_id));

drop index idx_community_lower_actor_id;
