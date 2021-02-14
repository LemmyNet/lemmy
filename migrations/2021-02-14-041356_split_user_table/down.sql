-- user_ table
-- Drop views
drop view person_alias_1, person_alias_2;

-- Rename indexes
alter index person__pkey rename to user__pkey;
alter index idx_person_actor_id rename to idx_user_actor_id;
alter index idx_person_inbox_url rename to idx_user_inbox_url;
alter index idx_person_lower_actor_id rename to idx_user_lower_actor_id;
alter index idx_person_published rename to idx_user_published;

-- Rename triggers
alter trigger site_aggregates_person_delete on person rename to site_aggregates_user_delete;
alter trigger site_aggregates_person_insert on person rename to site_aggregates_user_insert;
alter trigger person_aggregates_person on person rename to user_aggregates_user;

-- Rename the trigger functions
alter function site_aggregates_person_delete() rename to site_aggregates_user_delete;
alter function site_aggregates_person_insert() rename to site_aggregates_user_insert;
alter function person_aggregates_person() rename to user_aggregates_user;

-- Rename the table back to user_
alter table person rename to user_;

-- Add the columns back in
alter table user_
  add column password_encrypted text not null default 'changeme',
  add column email text,
  add column admin boolean default false not null,
  add column show_nsfw boolean default false not null,
  add column theme character varying(20) default 'darkly'::character varying not null,
  add column default_sort_type smallint default 0 not null,
  add column default_listing_type smallint default 1 not null,
  add column lang character varying(20) default 'browser'::character varying not null,
  add column show_avatars boolean default true not null,
  add column send_notifications_to_email boolean default false not null,
  add column matrix_user_id text;

-- Update the user_ table with the local_user data
update user_ u set
  password_encrypted = lu.password_encrypted,
  email = lu.email,
  admin = lu.admin,
  show_nsfw = lu.show_nsfw,
  theme = lu.theme,
  default_sort_type = lu.default_sort_type,
  default_listing_type = lu.default_listing_type,
  lang = lu.lang,
  show_avatars = lu.show_avatars,
  send_notifications_to_email = lu.send_notifications_to_email,
  matrix_user_id = lu.matrix_user_id
from local_user lu
where lu.user_id = u.id;

create view user_alias_1 as select * from user_;
create view user_alias_2 as select * from user_;

drop table local_user;

