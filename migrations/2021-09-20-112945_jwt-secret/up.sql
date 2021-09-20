-- generate a jwt secret with 62 possible characters and length 43.
-- this gives an entropy of 256 bits
-- log2(62^43) = 256

create table secrets(
  id serial primary key,
  jwt_secret varchar(43) not null
);
-- TODO: generate a random string from A-Za-z0-9, length 43, and insert
insert into secrets(jwt_secret) values('123');
