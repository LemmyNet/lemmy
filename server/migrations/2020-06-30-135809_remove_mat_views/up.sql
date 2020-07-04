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
    -- TODO test this. Also is it locking?
    delete from post_fast where creator_id = NEW.id;
    insert into post_fast select * from post_view where creator_id = NEW.id;

    -- TODO
    -- refresh materialized view concurrently comment_aggregates_mview; -- cause of bans
    -- refresh materialized view concurrently post_aggregates_mview; -- cause of user info changes

  ELSIF (TG_OP = 'INSERT') THEN
    insert into user_fast select * from user_view where id = NEW.id;
    -- Update all the fast views
    insert into community_fast select * from community_view where user_id = NEW.id;
    insert into post_fast select * from post_view where user_id = NEW.id;
    insert into comment_fast select * from comment_view where user_id = NEW.id;
  END IF;

  return null;
end $$;

-- Post

create table post_fast as select * from post_view;
alter table post_fast add column fast_id serial primary key;

create index idx_post_fast_user_id on post_fast (user_id);
create index idx_post_fast_id on post_fast (id);

-- For the hot rank resorting
create index idx_post_fast_hot_rank on post_fast (hot_rank);

-- This ones for the common case of null fetches
create index idx_post_fast_hot_rank_published_desc_user_null on post_fast (hot_rank desc, published desc) where user_id is null;

drop trigger refresh_post on post;

create trigger refresh_post
after insert or update or delete
on post
for each row
execute procedure refresh_post();

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
    delete from post_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from post_fast where id = OLD.id;
    insert into post_fast select * from post_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into post_fast select * from post_view where id = NEW.id;

    -- TODO Update the user fast table
    -- Update that users number of posts, post score
    -- delete from user_fast where id = NEW.creator_id;
    -- insert into user_fast select * from user_view where id = NEW.creator_id;

    -- Update the hot rank on the post table TODO hopefully this doesn't lock it.
    update post_fast set hot_rank = hot_rank(coalesce(score , 0), published) where hot_rank > 0 ;
  END IF;

  return null;
end $$;

-- Community
create table community_fast as select * from community_view;
alter table community_fast add column fast_id serial primary key;

create index idx_community_fast_id on community_fast (id);
create index idx_community_fast_user_id on community_fast (user_id);

drop trigger refresh_community on community;

create trigger refresh_community
after insert or update or delete
on community
for each row
execute procedure refresh_community();

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
    delete from community_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from community_fast where id = OLD.id;
    insert into community_fast select * from community_view where id = NEW.id;

    -- Update user view due to owner changes
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
    
    -- Update post view due to community changes
    delete from post_fast where community_id = NEW.id;
    insert into post_fast select * from post_view where community_id = NEW.id;

  -- TODO make sure this shows up in the users page ?
  ELSIF (TG_OP = 'INSERT') THEN
    insert into community_fast select * from community_view where id = NEW.id;
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

create table comment_fast as select * from comment_view;
alter table comment_fast add column fast_id serial primary key;

create index idx_comment_fast_user_id on comment_fast (user_id);
create index idx_comment_fast_id on comment_fast (id);

-- This ones for the common case of null fetches
create index idx_comment_fast_hot_rank_published_desc_user_null on comment_fast (hot_rank desc, published desc) where user_id is null;

drop trigger refresh_comment on comment;

create trigger refresh_comment
after insert or update or delete
on comment
for each row
execute procedure refresh_comment();

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
    -- delete from comment_fast where id = OLD.id;
  ELSIF (TG_OP = 'UPDATE') THEN
    -- delete from comment_fast where id = OLD.id;
    -- insert into comment_fast select * from comment_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into comment_fast select * from comment_view where id = NEW.id;

    -- Update user view due to comment count
    -- delete from user_fast where id = NEW.creator_id;
    -- insert into user_fast select * from user_view where id = NEW.creator_id;
    
    -- Update post view due to comment count
    -- delete from post_fast where id = NEW.post_id;
    -- insert into post_fast select * from post_view where id = NEW.post_id;

    -- Update community view due to comment count
    -- delete from community_fast as cf using post as p where cf.id = p.community_id and p.id = NEW.post_id;
    -- insert into community_fast select cv.* from community_view cv, post p where cv.id = p.community_id and p.id = NEW.post_id;

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

-- The reply view, referencing the fast table
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
from comment_fast cv, closereply
where closereply.id = cv.id
;

-- post_like
-- select id, score, my_vote from post_fast where id = 29 and user_id = 4;
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
  -- TODO possibly select from post_fast to get previous scores, instead of re-fetching the views?
  IF (TG_OP = 'DELETE') THEN
    delete from post_fast where id = OLD.post_id;
    insert into post_fast select * from post_view where id = OLD.post_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from post_fast where id = NEW.post_id;
    insert into post_fast select * from post_view where id = NEW.post_id;
  ELSIF (TG_OP = 'INSERT') THEN
    delete from post_fast where id = NEW.post_id;
    insert into post_fast select * from post_view where id = NEW.post_id;
  END IF;

  return null;
end $$;

drop trigger refresh_post_like on post_like;
create trigger refresh_post_like
after insert or update or delete
on post_like
for each row
execute procedure refresh_post_like();

create or replace function refresh_comment_like()
returns trigger language plpgsql
as $$
begin
  -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
  -- IF (TG_OP = 'DELETE') THEN
  --   delete from comment_fast where id = OLD.comment_id;
  --   insert into comment_fast select * from comment_view where id = OLD.comment_id;
  -- ELSIF (TG_OP = 'UPDATE') THEN
  --   delete from comment_fast where id = NEW.comment_id;
  --   insert into comment_fast select * from comment_view where id = NEW.comment_id;
  -- ELSIF (TG_OP = 'INSERT') THEN
  --   delete from comment_fast where id = NEW.comment_id;
  --   insert into comment_fast select * from comment_view where id = NEW.comment_id;
  -- END IF;

  return null;
end $$;

drop trigger refresh_comment_like on comment_like;
create trigger refresh_comment_like
after insert or update or delete
on comment_like
for each row
execute procedure refresh_comment_like();
