create table site_language (
  id serial primary key,
  site_id int references site on update cascade on delete cascade not null,
  language_id int references language on update cascade on delete cascade not null,
  unique (site_id, language_id)
);

create table community_language (
  id serial primary key,
  community_id int references community on update cascade on delete cascade not null,
  language_id int references language on update cascade on delete cascade not null,
  unique (community_id, language_id)
);
