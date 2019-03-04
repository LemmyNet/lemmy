create table user_ (
  id serial primary key,
  name varchar(20) not null,
  preferred_username varchar(20),
  password_encrypted text not null,
  email text,
  icon bytea,
  start_time timestamp not null default now()
);

