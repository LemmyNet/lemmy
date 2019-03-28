create table comment (
  id serial primary key,
  content text not null,
  attributed_to text not null,
  post_id int references post on update cascade on delete cascade not null,
  parent_id int references comment on update cascade on delete cascade,
  published timestamp not null default now(),
  updated timestamp
);

create table comment_like (
  id serial primary key,
  comment_id int references comment on update cascade on delete cascade not null,
  post_id int references post on update cascade on delete cascade not null,
  fedi_user_id text not null,
  score smallint not null, -- -1, or 1 for dislike, like, no row for no opinion
  published timestamp not null default now(),
  unique(comment_id, fedi_user_id)
);
