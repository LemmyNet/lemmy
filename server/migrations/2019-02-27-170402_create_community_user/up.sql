-- No support for types yet, so just do 0,1,2
-- create type community_user_type as enum ('creator', 'moderator', 'user');

create table community_user (
  id serial primary key,
  fedi_user_id varchar(100) not null,
  community_id int references community on update cascade on delete cascade,
  community_user_type smallint not null default 2,
  starttime timestamp not null default now()
)
