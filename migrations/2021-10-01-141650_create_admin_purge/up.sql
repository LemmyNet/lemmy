-- Add the admin_purge tables

create table admin_purge_person (
  id serial primary key,
  admin_person_id int references person on update cascade on delete cascade not null,
  reason text,
  when_ timestamp not null default now()
);

create table admin_purge_community (
  id serial primary key,
  admin_person_id int references person on update cascade on delete cascade not null,
  reason text,
  when_ timestamp not null default now()
);

create table admin_purge_post (
  id serial primary key,
  admin_person_id int references person on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  reason text,
  when_ timestamp not null default now()
);

create table admin_purge_comment (
  id serial primary key,
  admin_person_id int references person on update cascade on delete cascade not null,
  post_id int references post on update cascade on delete cascade not null,
  reason text,
  when_ timestamp not null default now()
);
