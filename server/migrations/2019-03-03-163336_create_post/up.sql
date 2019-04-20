create table post (
  id serial primary key,
  name varchar(100) not null,
  url text, -- These are both optional, a post can just have a title
  body text,
  creator_id int references user_ on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  removed boolean default false not null,
  locked boolean default false not null,
  published timestamp not null default now(),
  updated timestamp
);

create table post_like (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  user_id int references user_ on update cascade on delete cascade not null,
  score smallint not null, -- -1, or 1 for dislike, like, no row for no opinion
  published timestamp not null default now(),
  unique(post_id, user_id)
);

create table post_saved (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(post_id, user_id)
);

create table post_read (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  user_id int references user_ on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(post_id, user_id)
);
