

-- Drop the mviews
drop view post_mview;
drop materialized view user_mview;
drop view community_mview;
drop materialized view private_message_mview;
drop view user_mention_mview;
drop view reply_view;
drop view comment_mview;
drop materialized view post_aggregates_mview;
drop materialized view community_aggregates_mview;
drop materialized view comment_aggregates_mview;

-- User
create table user_fast as select * from user_view;
alter table user_fast add column fast_id serial primary key;

create index idx_user_fast_id on user_fast (id);

drop trigger refresh_user on user_;

create trigger refresh_user
after insert or update or delete
on user_
for each row
execute procedure refresh_user();

-- Sample insert 
-- insert into user_(name, password_encrypted) values ('test_name', 'bleh');
-- Sample delete
-- delete from user_ where name like 'test_name';
-- Sample update
-- update user_ set avatar = 'hai'  where name like 'test_name';
create or replace function refresh_user()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from user_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from user_fast where id = OLD.id;
    insert into user_fast select * from user_view where id = NEW.id;
    
    -- Refresh post_fast, cause of user info changes
    -- TODO test this (for example a banned user). Also is it locking?
    delete from post_aggregates_fast where creator_id = NEW.id;
    insert into post_aggregates_fast select * from post_aggregates_view where creator_id = NEW.id;

    delete from comment_aggregates_fast where creator_id = NEW.id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where creator_id = NEW.id;

  ELSIF (TG_OP = 'INSERT') THEN
    insert into user_fast select * from user_view where id = NEW.id;
  END IF;

  return null;
end $$;

-- Post

create table post_aggregates_fast as select * from post_aggregates_view;
alter table post_aggregates_fast add column fast_id serial primary key;

create index idx_post_aggregates_fast_id on post_aggregates_fast (id);

-- For the hot rank resorting
create index idx_post_aggregates_fast_hot_rank on post_aggregates_fast (hot_rank desc);
create index idx_post_aggregates_fast_activity on post_aggregates_fast (newest_activity_time desc);

create view post_fast_view as 
with all_post as (
  select
  pa.*
  from post_aggregates_fast pa
)
select
ap.*,
u.id as user_id,
coalesce(pl.score, 0) as my_vote,
(select cf.id::bool from community_follower cf where u.id = cf.user_id and cf.community_id = ap.community_id) as subscribed,
(select pr.id::bool from post_read pr where u.id = pr.user_id and pr.post_id = ap.id) as read,
(select ps.id::bool from post_saved ps where u.id = ps.user_id and ps.post_id = ap.id) as saved
from user_ u
cross join all_post ap
left join post_like pl on u.id = pl.user_id and ap.id = pl.post_id

union all

select 
ap.*,
null as user_id,
null as my_vote,
null as subscribed,
null as read,
null as saved
from all_post ap
;

drop trigger refresh_post on post;

create trigger refresh_post
after insert or update or delete
on post
for each row
execute procedure refresh_post();

-- Sample select
-- select id, name from post_fast_view where name like 'test_post' and user_id is null;
-- Sample insert 
-- insert into post(name, creator_id, community_id) values ('test_post', 2, 2);
-- Sample delete
-- delete from post where name like 'test_post';
-- Sample update
-- update post set community_id = 4  where name like 'test_post';
create or replace function refresh_post()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from post_aggregates_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from post_aggregates_fast where id = OLD.id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.id;

    -- Update that users number of posts, post score
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;

    -- Update the hot rank on the post table TODO hopefully this doesn't lock it.
    -- TODO this might not correctly update it, using a 1 week interval
    update post_aggregates_fast as paf
    set hot_rank = pav.hot_rank 
    from post_aggregates_view as pav
    where paf.id = pav.id  and (pav.published > ('now'::timestamp - '1 week'::interval));
  END IF;

  return null;
end $$;

-- Community
create table community_aggregates_fast as select * from community_aggregates_view;
alter table community_aggregates_fast add column fast_id serial primary key;

create index idx_community_aggregates_fast_id on community_aggregates_fast (id);

create view community_fast_view as
with all_community as
(
  select
  ca.*
  from community_aggregates_fast ca
)

select
ac.*,
u.id as user_id,
(select cf.id::boolean from community_follower cf where u.id = cf.user_id and ac.id = cf.community_id) as subscribed
from user_ u
cross join all_community ac

union all

select 
ac.*,
null as user_id,
null as subscribed
from all_community ac;

drop trigger refresh_community on community;

create trigger refresh_community
after insert or update or delete
on community
for each row
execute procedure refresh_community();

-- Sample select
-- select * from community_fast_view where name like 'test_community_name' and user_id is null;
-- Sample insert 
-- insert into community(name, title, category_id, creator_id) values ('test_community_name', 'test_community_title', 1, 2);
-- Sample delete
-- delete from community where name like 'test_community_name';
-- Sample update
-- update community set title = 'test_community_title_2'  where name like 'test_community_name';
create or replace function refresh_community()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from community_aggregates_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from community_aggregates_fast where id = OLD.id;
    insert into community_aggregates_fast select * from community_aggregates_view where id = NEW.id;

    -- Update user view due to owner changes
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
    
    -- Update post view due to community changes
    delete from post_aggregates_fast where community_id = NEW.id;
    insert into post_aggregates_fast select * from post_aggregates_view where community_id = NEW.id;

  -- TODO make sure this shows up in the users page ?
  ELSIF (TG_OP = 'INSERT') THEN
    insert into community_aggregates_fast select * from community_aggregates_view where id = NEW.id;
  END IF;

  return null;
end $$;

-- Private message

create table private_message_fast as select * from private_message_view;
alter table private_message_fast add column fast_id serial primary key;

create index idx_private_message_fast_id on private_message_fast (id);

drop trigger refresh_private_message on private_message;

create trigger refresh_private_message
after insert or update or delete
on private_message
for each row
execute procedure refresh_private_message();

-- Sample insert 
-- insert into private_message(creator_id, recipient_id, content) values (2, 3, 'test_private_message');
-- Sample delete
-- delete from private_message where content like 'test_private_message';
-- Sample update
-- update private_message set ap_id = 'test' where content like 'test_private_message';
create or replace function refresh_private_message()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from private_message_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from private_message_fast where id = OLD.id;
    insert into private_message_fast select * from private_message_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into private_message_fast select * from private_message_view where id = NEW.id;
  END IF;

  return null;
end $$;

-- Comment

create table comment_aggregates_fast as select * from comment_aggregates_view;
alter table comment_aggregates_fast add column fast_id serial primary key;

create index idx_comment_aggregates_fast_id on comment_aggregates_fast (id);

create view comment_fast_view as
with all_comment as
(
  select
  ca.*
  from comment_aggregates_fast ca
)

select
ac.*,
u.id as user_id,
coalesce(cl.score, 0) as my_vote,
(select cf.id::boolean from community_follower cf where u.id = cf.user_id and ac.community_id = cf.community_id) as subscribed,
(select cs.id::bool from comment_saved cs where u.id = cs.user_id and cs.comment_id = ac.id) as saved
from user_ u
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id

union all

select 
    ac.*,
    null as user_id, 
    null as my_vote,
    null as subscribed,
    null as saved
from all_comment ac
;

-- Do the reply_view referencing the comment_fast_view
create view reply_fast_view as 
with closereply as (
    select 
    c2.id, 
    c2.creator_id as sender_id, 
    c.creator_id as recipient_id
    from comment c
    inner join comment c2 on c.id = c2.parent_id
    where c2.creator_id != c.creator_id
    -- Do union where post is null
    union
    select
    c.id,
    c.creator_id as sender_id,
    p.creator_id as recipient_id
    from comment c, post p
    where c.post_id = p.id and c.parent_id is null and c.creator_id != p.creator_id
)
select cv.*,
closereply.recipient_id
from comment_fast_view cv, closereply
where closereply.id = cv.id
;

drop trigger refresh_comment on comment;

create trigger refresh_comment
after insert or update or delete
on comment
for each row
execute procedure refresh_comment();

-- Sample select
-- select * from comment_fast_view where content = 'test_comment' and user_id is null;
-- Sample insert 
-- insert into comment(creator_id, post_id, content) values (2, 2, 'test_comment');
-- Sample delete
-- delete from comment where content like 'test_comment';
-- Sample update
-- update comment set removed = true where content like 'test_comment';
create or replace function refresh_comment()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from comment_aggregates_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from comment_aggregates_fast where id = OLD.id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.id;

    -- Update user view due to comment count
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
    
    -- Update post view due to comment count, new comment activity time, but only on new posts
    delete from post_aggregates_fast where id = NEW.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.post_id;

    -- Force the hot rank as zero on week-older posts
    update post_aggregates_fast as paf
    set hot_rank = 0
    where paf.id = NEW.post_id and (paf.published < ('now'::timestamp - '1 week'::interval));

    -- Update community view due to comment count
    delete from community_aggregates_fast as cf using post as p where cf.id = p.community_id and p.id = NEW.post_id;
    insert into community_aggregates_fast select cv.* from community_aggregates_view cv, post p where cv.id = p.community_id and p.id = NEW.post_id;

  END IF;

  return null;
end $$;

-- User mention

create table user_mention_fast as select * from user_mention_view;
alter table user_mention_fast add column fast_id serial primary key;

create index idx_user_mention_fast_user_id on user_mention_fast (user_id);
create index idx_user_mention_fast_id on user_mention_fast (id);

-- Sample insert 
-- insert into user_mention(recipient_id, comment_id) values (2, 4);
-- Sample delete
-- delete from user_mention where recipient_id = 2 and comment_id = 4;
-- Sample update
-- update user_mention set read = true where recipient_id = 2 and comment_id = 4;
create or replace function refresh_user_mention()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from user_mention_fast where id = OLD.comment_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from user_mention_fast where id = OLD.comment_id;
    insert into user_mention_fast select * from user_mention_view where id = NEW.comment_id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into user_mention_fast select * from user_mention_view where id = NEW.comment_id;

  END IF;

  return null;
end $$;

create trigger refresh_user_mention
after insert or update or delete
on user_mention
for each row
execute procedure refresh_user_mention();

-- post_like
-- select id, score, my_vote from post_fast_view where id = 29 and user_id = 4;
-- Sample insert 
-- insert into post_like(user_id, post_id, score) values (4, 29, 1);
-- Sample delete
-- delete from post_like where user_id = 4 and post_id = 29;
-- Sample update
-- update post_like set score = -1 where user_id = 4 and post_id = 29;

-- TODO test this a LOT
create or replace function refresh_post_like()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    delete from post_aggregates_fast where id = OLD.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = OLD.post_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from post_aggregates_fast where id = NEW.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.post_id;
  ELSIF (TG_OP = 'INSERT') THEN
    delete from post_aggregates_fast where id = NEW.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.post_id;
  END IF;

  return null;
end $$;

drop trigger refresh_post_like on post_like;
create trigger refresh_post_like
after insert or update or delete
on post_like
for each row
execute procedure refresh_post_like();

-- comment_like
-- select id, score, my_vote from comment_fast_view where id = 29 and user_id = 4;
-- Sample insert 
-- insert into comment_like(user_id, comment_id, post_id, score) values (4, 29, 51, 1);
-- Sample delete
-- delete from comment_like where user_id = 4 and comment_id = 29;
-- Sample update
-- update comment_like set score = -1 where user_id = 4 and comment_id = 29;
create or replace function refresh_comment_like()
returns trigger language plpgsql
as $$
begin
  -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
  IF (TG_OP = 'DELETE') THEN
    delete from comment_aggregates_fast where id = OLD.comment_id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = OLD.comment_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from comment_aggregates_fast where id = NEW.comment_id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.comment_id;
  ELSIF (TG_OP = 'INSERT') THEN
    delete from comment_aggregates_fast where id = NEW.comment_id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.comment_id;
  END IF;

  return null;
end $$;

drop trigger refresh_comment_like on comment_like;
create trigger refresh_comment_like
after insert or update or delete
on comment_like
for each row
execute procedure refresh_comment_like();
