-- The username index
drop index idx_user_name_lower_actor_id;
create unique index idx_user_name_lower on user_ (lower(name));

