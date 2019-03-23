create table user_ (
  id serial primary key,
  name varchar(20) not null unique,
  preferred_username varchar(20),
  password_encrypted text not null,
  email text unique,
  icon bytea,
  published timestamp not null default now(),
  updated timestamp
)
