create table community (
  id serial primary key,
  name varchar(20) not null unique,
  published timestamp not null default now(),
  updated timestamp
);

create table community_user (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  fedi_user_id text not null,
  published timestamp not null default now()
);

create table community_follower (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  fedi_user_id text not null,
  published timestamp not null default now()
);

insert into community (name) values ('main');
