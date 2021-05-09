create table person_block (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  recipient_id int references person on update cascade on delete cascade not null,
  published timestamp not null default now()
);

create table community_block (
  id serial primary key,
  person_id int references person on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  published timestamp not null default now()
);
