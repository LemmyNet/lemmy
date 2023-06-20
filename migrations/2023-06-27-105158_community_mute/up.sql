create table community_mute (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  published timestamp not null default now(),
  unique(person_id, community_id)
);

create index idx_community_mute_community on community_mute (community_id);
create index idx_community_mute_person on community_mute (person_id);
