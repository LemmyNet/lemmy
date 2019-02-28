create table user_ (
  id serial primary key,
  name varchar(20) not null,
  password_encrypted varchar(200) not null,
  email varchar(200),
  icon bytea,
  startTime timestamp not null default now()
)
