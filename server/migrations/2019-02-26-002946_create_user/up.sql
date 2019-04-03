create table user_ (
  id serial primary key,
  name varchar(20) not null,
  fedi_name varchar(40) not null,
  preferred_username varchar(20),
  password_encrypted text not null,
  email text unique,
  icon bytea,
  published timestamp not null default now(),
  updated timestamp,
  unique(name, fedi_name)
);

insert into user_ (name, fedi_name, password_encrypted) values ('admin', 'TBD', 'TBD');
