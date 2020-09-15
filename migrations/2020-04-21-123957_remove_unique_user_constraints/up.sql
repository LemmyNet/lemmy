drop index idx_user_name_lower;
create unique index idx_user_name_lower_actor_id on user_ (lower(name), lower(actor_id));
