-- Drop the 2 views user_alias_1, user_alias_2
drop view user_alias_1, user_alias_2;

-- rename the user_ table to person
alter table user_ rename to person;

-- create a new table local_user
create table local_user (
  id serial primary key,
  user_id int references person on update cascade on delete cascade not null,
  password_encrypted text not null,
  email text,
  admin boolean default false not null,
  show_nsfw boolean default false not null,
  theme character varying(20) default 'darkly'::character varying not null,
  default_sort_type smallint default 0 not null,
  default_listing_type smallint default 1 not null,
  lang character varying(20) default 'browser'::character varying not null,
  show_avatars boolean default true not null,
  send_notifications_to_email boolean default false not null,
  matrix_user_id text,
  unique (user_id)
);

-- Copy the local users over to the new table
insert into local_user 
(
  user_id,
  password_encrypted,
  email,
  admin,
  show_nsfw,
  theme,
  default_sort_type,
  default_listing_type,
  lang,
  show_avatars,
  send_notifications_to_email,
  matrix_user_id
)
select
  id,
  password_encrypted,
  email,
  admin,
  show_nsfw,
  theme,
  default_sort_type,
  default_listing_type,
  lang,
  show_avatars,
  send_notifications_to_email,
  matrix_user_id
from person
where local = true;

-- Drop those columns from person
alter table person 
  drop column password_encrypted,
  drop column email,
  drop column admin,
  drop column show_nsfw,
  drop column theme,
  drop column default_sort_type,
  drop column default_listing_type,
  drop column lang,
  drop column show_avatars,
  drop column send_notifications_to_email,
  drop column matrix_user_id;

-- Rename indexes
alter index user__pkey rename to person__pkey;
alter index idx_user_actor_id rename to idx_person_actor_id;
alter index idx_user_inbox_url rename to idx_person_inbox_url;
alter index idx_user_lower_actor_id rename to idx_person_lower_actor_id;
alter index idx_user_published rename to idx_person_published;

-- Rename triggers
alter trigger site_aggregates_user_delete on person rename to site_aggregates_person_delete;
alter trigger site_aggregates_user_insert on person rename to site_aggregates_person_insert;
alter trigger user_aggregates_user on person rename to person_aggregates_person;

-- Rename the trigger functions
alter function site_aggregates_user_delete() rename to site_aggregates_person_delete;
alter function site_aggregates_user_insert() rename to site_aggregates_person_insert;
alter function user_aggregates_user() rename to person_aggregates_person;

-- Create views
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;

-- Rename every user_id column to person_id

