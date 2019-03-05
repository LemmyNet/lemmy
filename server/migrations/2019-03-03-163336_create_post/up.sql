create table post (
  id serial primary key,
  name varchar(100) not null,
  url text not null,
  attributed_to text not null,
  published timestamp not null default now(),
  updated timestamp
);

create table post_like (
  id serial primary key,
  fedi_user_id text not null,
  post_id int references post on update cascade on delete cascade,
  published timestamp not null default now()
);

create table post_dislike (
  id serial primary key,
  fedi_user_id text not null,
  post_id int references post on update cascade on delete cascade,
  published timestamp not null default now()
);

