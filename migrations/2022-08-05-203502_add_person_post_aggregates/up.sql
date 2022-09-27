-- This table stores the # of read comments for a person, on a post
-- It can then be joined to post_aggregates to get an unread count:
-- unread = post_aggregates.comments - person_post_aggregates.read_comments
create table person_post_aggregates(
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  post_id int references post on update cascade on delete cascade not null,
  read_comments bigint not null default 0,
  published timestamp not null default now(),
  unique(person_id, post_id)
);
