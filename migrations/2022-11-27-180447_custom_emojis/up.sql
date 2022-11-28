create table custom_emoji (
  id serial primary key,
  local_site_id int references local_site on update cascade on delete cascade not null,
  shortcode varchar(128) not null UNIQUE,
  image_url text not null UNIQUE,
  alt_text text not null,
  category text not null,
  published timestamp without time zone default now() not null,
  updated timestamp without time zone
);

create table custom_emoji_keyword (
  id serial primary key,
  custom_emoji_id int references custom_emoji on update cascade on delete cascade not null,
  keyword varchar(128) not null
);