create table community (
  id serial primary key,
  name varchar(20) not null unique,
  creator_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  updated timestamp
);

create table community_moderator (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now()
);

create table community_follower (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now()
);

insert into community (name, creator_id) values ('main', 1);
