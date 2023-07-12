create table community_hide_from_feeds (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(person_id, community_id)
);

create index idx_community_hide_from_feeds_community on community_hide_from_feeds (community_id);
create index idx_community_hide_from_feeds_person on community_hide_from_feeds (person_id);
