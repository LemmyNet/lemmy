create table post (
  id serial primary key,
  name varchar(100) not null,
  url text, -- These are both optional, a post can just have a title
  body text,
  attributed_to text not null,
  community_id int references community on update cascade on delete cascade not null,
  published timestamp not null default now(),
  updated timestamp
);

create table post_like (
  id serial primary key,
  post_id int references post on update cascade on delete cascade not null,
  fedi_user_id text not null,
  score smallint not null, -- -1, or 1 for dislike, like, no row for no opinion
  published timestamp not null default now(),
  unique(post_id, fedi_user_id)
);

