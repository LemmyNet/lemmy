create table tagline (
  id serial primary key,
  local_site_id int references local_site on update cascade on delete cascade not null,
  content text not null,
  published timestamp without time zone default now() not null,
  updated timestamp without time zone
);