create table community (
  id serial primary key,
  name varchar(20) not null,
  start_time timestamp not null default now()
);

create table community_user (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  fedi_user_id text not null,
  start_time timestamp not null default now()
);

create table community_follower (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  fedi_user_id text not null,
  start_time timestamp not null default now()
);
