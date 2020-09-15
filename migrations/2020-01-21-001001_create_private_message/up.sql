-- Creating private message
create table private_message (
  id serial primary key,
  creator_id int references user_ on update cascade on delete cascade not null,
  recipient_id int references user_ on update cascade on delete cascade not null,
  content text not null,
  deleted boolean default false not null,
  read boolean default false not null,
  published timestamp not null default now(),
  updated timestamp
);

-- Create the view and materialized view which has the avatar and creator name
create view private_message_view as 
select        
pm.*,
u.name as creator_name,
u.avatar as creator_avatar,
u2.name as recipient_name,
u2.avatar as recipient_avatar
from private_message pm
inner join user_ u on u.id = pm.creator_id
inner join user_ u2 on u2.id = pm.recipient_id;

create materialized view private_message_mview as select * from private_message_view;

create unique index idx_private_message_mview_id on private_message_mview (id);

-- Create the triggers
create or replace function refresh_private_message()
returns trigger language plpgsql
as $$
begin
  refresh materialized view concurrently private_message_mview;
  return null;
end $$;

create trigger refresh_private_message
after insert or update or delete or truncate
on private_message
for each statement
execute procedure refresh_private_message();

-- Update user to include matrix id
alter table user_ add column matrix_user_id text unique;

drop view user_view cascade;
create view user_view as 
select 
u.id,
u.name,
u.avatar,
u.email,
u.matrix_user_id,
u.fedi_name,
u.admin,
u.banned,
u.show_avatars,
u.send_notifications_to_email,
u.published,
(select count(*) from post p where p.creator_id = u.id) as number_of_posts,
(select coalesce(sum(score), 0) from post p, post_like pl where u.id = p.creator_id and p.id = pl.post_id) as post_score,
(select count(*) from comment c where c.creator_id = u.id) as number_of_comments,
(select coalesce(sum(score), 0) from comment c, comment_like cl where u.id = c.creator_id and c.id = cl.comment_id) as comment_score
from user_ u;

create materialized view user_mview as select * from user_view;

create unique index idx_user_mview_id on user_mview (id);

-- This is what a group pm table would look like
-- Not going to do it now because of the complications
-- 
-- create table private_message (
--   id serial primary key,
--   creator_id int references user_ on update cascade on delete cascade not null,
--   content text not null,
--   deleted boolean default false not null,
--   published timestamp not null default now(),
--   updated timestamp
-- );
-- 
-- create table private_message_recipient (
--   id serial primary key,
--   private_message_id int references private_message on update cascade on delete cascade not null,
--   recipient_id int references user_ on update cascade on delete cascade not null,
--   read boolean default false not null,
--   published timestamp not null default now(),
--   unique(private_message_id, recipient_id)
-- )
