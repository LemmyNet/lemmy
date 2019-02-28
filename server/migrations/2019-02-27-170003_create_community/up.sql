create table community (
  id serial primary key,
  name varchar(20) not null,
  starttime timestamp not null default now()
)
