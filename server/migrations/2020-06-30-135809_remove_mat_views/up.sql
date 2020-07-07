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
drop trigger refresh_private_message on private_message;

-- User
drop view user_view;
create view user_view as
select 
	u.id,
  u.actor_id,
	u.name,
	u.avatar,
	u.email,
	u.matrix_user_id,
  u.bio,
  u.local,
	u.admin,
	u.banned,
	u.show_avatars,
	u.send_notifications_to_email,
	u.published,
	coalesce(pd.posts, 0) as number_of_posts,
	coalesce(pd.score, 0) as post_score,
	coalesce(cd.comments, 0) as number_of_comments,
	coalesce(cd.score, 0) as comment_score
from user_ u
left join (
    select
        p.creator_id as creator_id,
        count(distinct p.id) as posts,
        sum(pl.score) as score
    from post p
    join post_like pl on p.id = pl.post_id
    group by p.creator_id
) pd on u.id = pd.creator_id
left join (
    select
        c.creator_id,
        count(distinct c.id) as comments,
        sum(cl.score) as score
    from comment c
    join comment_like cl on c.id = cl.comment_id
    group by c.creator_id
) cd on u.id = cd.creator_id;


create table user_fast as select * from user_view;
alter table user_fast add primary key (id);

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
-- Redoing the views : Credit eiknat
drop view post_view;
drop view post_aggregates_view;

create view post_aggregates_view as
select
	p.*,
	-- creator details
	u.actor_id as creator_actor_id,
	u."local" as creator_local,
	u."name" as creator_name,
	u.avatar as creator_avatar,
  u.banned as banned,
  cb.id::bool as banned_from_community,
	-- community details
	c.actor_id as community_actor_id,
	c."local" as community_local,
	c."name" as community_name,
	c.removed as community_removed,
	c.deleted as community_deleted,
	c.nsfw as community_nsfw,
	-- post score data/comment count
	coalesce(ct.comments, 0) as number_of_comments,
	coalesce(pl.score, 0) as score,
	coalesce(pl.upvotes, 0) as upvotes,
	coalesce(pl.downvotes, 0) as downvotes,
	hot_rank(
		coalesce(pl.score , 0), (
			case
				when (p.published < ('now'::timestamp - '1 month'::interval))
				then p.published
				else greatest(ct.recent_comment_time, p.published)
			end
		)
	) as hot_rank,
	(
		case
			when (p.published < ('now'::timestamp - '1 month'::interval))
			then p.published
			else greatest(ct.recent_comment_time, p.published)
		end
	) as newest_activity_time
from post p
left join user_ u on p.creator_id = u.id
left join community_user_ban cb on p.creator_id = cb.user_id and p.community_id = cb.community_id
left join community c on p.community_id = c.id
left join (
	select
		post_id,
		count(*) as comments,
		max(published) as recent_comment_time
	from comment
	group by post_id
) ct on ct.post_id = p.id
left join (
	select
		post_id,
		sum(score) as score,
		sum(score) filter (where score = 1) as upvotes,
		-sum(score) filter (where score = -1) as downvotes
	from post_like
	group by post_id
) pl on pl.post_id = p.id
order by p.id;

create view post_view as
select
	pav.*,
	us.id as user_id,
	us.user_vote as my_vote,
	us.is_subbed::bool as subscribed,
	us.is_read::bool as read,
	us.is_saved::bool as saved
from post_aggregates_view pav
cross join lateral (
	select
		u.id,
		coalesce(cf.community_id, 0) as is_subbed,
		coalesce(pr.post_id, 0) as is_read,
		coalesce(ps.post_id, 0) as is_saved,
		coalesce(pl.score, 0) as user_vote
	from user_ u
	left join community_user_ban cb on u.id = cb.user_id and cb.community_id = pav.community_id
	left join community_follower cf on u.id = cf.user_id and cf.community_id = pav.community_id
	left join post_read pr on u.id = pr.user_id and pr.post_id = pav.id
	left join post_saved ps on u.id = ps.user_id and ps.post_id = pav.id
	left join post_like pl on u.id = pl.user_id and pav.id = pl.post_id
) as us

union all

select 
pav.*,
null as user_id,
null as my_vote,
null as subscribed,
null as read,
null as saved
from post_aggregates_view pav;

-- The post fast table
create table post_aggregates_fast as select * from post_aggregates_view;
alter table post_aggregates_fast add primary key (id);

-- For the hot rank resorting
create index idx_post_aggregates_fast_hot_rank_published on post_aggregates_fast (hot_rank desc, published desc);

create view post_fast_view as 
select
	pav.*,
	us.id as user_id,
	us.user_vote as my_vote,
	us.is_subbed::bool as subscribed,
	us.is_read::bool as read,
	us.is_saved::bool as saved
from post_aggregates_fast pav
cross join lateral (
	select
		u.id,
		coalesce(cf.community_id, 0) as is_subbed,
		coalesce(pr.post_id, 0) as is_read,
		coalesce(ps.post_id, 0) as is_saved,
		coalesce(pl.score, 0) as user_vote
	from user_ u
	left join community_user_ban cb on u.id = cb.user_id and cb.community_id = pav.community_id
	left join community_follower cf on u.id = cf.user_id and cf.community_id = pav.community_id
	left join post_read pr on u.id = pr.user_id and pr.post_id = pav.id
	left join post_saved ps on u.id = ps.user_id and ps.post_id = pav.id
	left join post_like pl on u.id = pl.user_id and pav.id = pl.post_id
) as us

union all

select 
pav.*,
null as user_id,
null as my_vote,
null as subscribed,
null as read,
null as saved
from post_aggregates_fast pav;

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

    -- Update community number of posts
    update community_aggregates_fast set number_of_posts = number_of_posts - 1 where id = OLD.community_id;
  ELSIF (TG_OP = 'UPDATE') THEN
    delete from post_aggregates_fast where id = OLD.id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.id;

    -- Update that users number of posts, post score
    delete from user_fast where id = NEW.creator_id;
    insert into user_fast select * from user_view where id = NEW.creator_id;
  
    -- Update community number of posts
    update community_aggregates_fast set number_of_posts = number_of_posts + 1 where id = NEW.community_id;

    -- Update the hot rank on the post table
    -- TODO this might not correctly update it, using a 1 week interval
    update post_aggregates_fast as paf
    set hot_rank = pav.hot_rank 
    from post_aggregates_view as pav
    where paf.id = pav.id  and (pav.published > ('now'::timestamp - '1 week'::interval));
  END IF;

  return null;
end $$;

-- Community
-- Redoing the views : Credit eiknat
drop view community_moderator_view;
drop view community_follower_view;
drop view community_user_ban_view;
drop view community_view;
drop view community_aggregates_view;

create view community_aggregates_view as
select 
    c.id,
    c.name,
    c.title,
    c.description,
    c.category_id,
    c.creator_id,
    c.removed,
    c.published,
    c.updated,
    c.deleted,
    c.nsfw,
    c.actor_id,
    c.local,
    c.last_refreshed_at,
    u.actor_id as creator_actor_id,
    u.local as creator_local,
    u.name as creator_name,
    u.avatar as creator_avatar,
    cat.name as category_name,
    coalesce(cf.subs, 0) as number_of_subscribers,
    coalesce(cd.posts, 0) as number_of_posts,
    coalesce(cd.comments, 0) as number_of_comments,
    hot_rank(cf.subs, c.published) as hot_rank
from community c
left join user_ u on c.creator_id = u.id
left join category cat on c.category_id = cat.id
left join (
    select
        p.community_id,
        count(distinct p.id) as posts,
        count(distinct ct.id) as comments
    from post p
    join comment ct on p.id = ct.post_id
    group by p.community_id
) cd on cd.community_id = c.id
left join (
    select
        community_id,
        count(*) as subs 
    from community_follower
    group by community_id 
) cf on cf.community_id = c.id;

create view community_view as
select
    cv.*,
    us.user as user_id,
    us.is_subbed::bool as subscribed
from community_aggregates_view cv
cross join lateral (
	select
		u.id as user,
		coalesce(cf.community_id, 0) as is_subbed
	from user_ u
	left join community_follower cf on u.id = cf.user_id and cf.community_id = cv.id
) as us

union all

select 
    cv.*,
    null as user_id,
    null as subscribed
from community_aggregates_view cv;

create view community_moderator_view as
select
    cm.*,
    u.actor_id as user_actor_id,
    u.local as user_local,
    u.name as user_name,
    u.avatar as avatar,
    c.actor_id as community_actor_id,
    c.local as community_local,
    c.name as community_name
from community_moderator cm
left join user_ u on cm.user_id = u.id
left join community c on cm.community_id = c.id;

create view community_follower_view as
select
    cf.*,
    u.actor_id as user_actor_id,
    u.local as user_local,
    u.name as user_name,
    u.avatar as avatar,
    c.actor_id as community_actor_id,
    c.local as community_local,
    c.name as community_name
from community_follower cf
left join user_ u on cf.user_id = u.id
left join community c on cf.community_id = c.id;

create view community_user_ban_view as
select
    cb.*,
    u.actor_id as user_actor_id,
    u.local as user_local,
    u.name as user_name,
    u.avatar as avatar,
    c.actor_id as community_actor_id,
    c.local as community_local,
    c.name as community_name
from community_user_ban cb
left join user_ u on cb.user_id = u.id
left join community c on cb.community_id = c.id;

-- The community fast table

create table community_aggregates_fast as select * from community_aggregates_view;
alter table community_aggregates_fast add primary key (id);

create view community_fast_view as
select
ac.*,
u.id as user_id,
(select cf.id::boolean from community_follower cf where u.id = cf.user_id and ac.id = cf.community_id) as subscribed
from user_ u
cross join (
  select
  ca.*
  from community_aggregates_fast ca
) ac

union all

select 
caf.*,
null as user_id,
null as subscribed
from community_aggregates_fast caf;

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

-- Comment

drop view user_mention_view;
drop view comment_view;
drop view comment_aggregates_view;

create view comment_aggregates_view as 
select
	ct.*,
	-- community details
	p.community_id,
	c.actor_id as community_actor_id,
	c."local" as community_local,
	c."name" as community_name,
	-- creator details
	u.banned as banned,
  coalesce(cb.id, 0)::bool as banned_from_community,
	u.actor_id as creator_actor_id,
	u.local as creator_local,
	u.name as creator_name,
	u.avatar as creator_avatar,
	-- score details
	coalesce(cl.total, 0) as score,
	coalesce(cl.up, 0) as upvotes,
	coalesce(cl.down, 0) as downvotes,
	hot_rank(coalesce(cl.total, 0), ct.published) as hot_rank
from comment ct
left join post p on ct.post_id = p.id
left join community c on p.community_id = c.id
left join user_ u on ct.creator_id = u.id 
left join community_user_ban cb on ct.creator_id = cb.user_id and p.id = ct.post_id and p.community_id = cb.community_id
left join (
	select
		l.comment_id as id,
		sum(l.score) as total,
		count(case when l.score = 1 then 1 else null end) as up,
		count(case when l.score = -1 then 1 else null end) as down
	from comment_like l
	group by comment_id
) as cl on cl.id = ct.id;

create or replace view comment_view as (
select
	cav.*,
  us.user_id as user_id,
  us.my_vote as my_vote,
  us.is_subbed::bool as subscribed,
  us.is_saved::bool as saved
from comment_aggregates_view cav
cross join lateral (
	select
		u.id as user_id,
		coalesce(cl.score, 0) as my_vote,
    coalesce(cf.id, 0) as is_subbed,
    coalesce(cs.id, 0) as is_saved
	from user_ u
	left join comment_like cl on u.id = cl.user_id and cav.id = cl.comment_id
	left join comment_saved cs on u.id = cs.user_id and cs.comment_id = cav.id
	left join community_follower cf on u.id = cf.user_id and cav.community_id = cf.community_id
) as us

union all 

select 
    cav.*,
    null as user_id, 
    null as my_vote,
    null as subscribed,
    null as saved
from comment_aggregates_view cav
);

-- The fast view
create table comment_aggregates_fast as select * from comment_aggregates_view;
alter table comment_aggregates_fast add primary key (id);

create view comment_fast_view as
select
	cav.*,
  us.user_id as user_id,
  us.my_vote as my_vote,
  us.is_subbed::bool as subscribed,
  us.is_saved::bool as saved
from comment_aggregates_fast cav
cross join lateral (
	select
		u.id as user_id,
		coalesce(cl.score, 0) as my_vote,
    coalesce(cf.id, 0) as is_subbed,
    coalesce(cs.id, 0) as is_saved
	from user_ u
	left join comment_like cl on u.id = cl.user_id and cav.id = cl.comment_id
	left join comment_saved cs on u.id = cs.user_id and cs.comment_id = cav.id
	left join community_follower cf on u.id = cf.user_id and cav.community_id = cf.community_id
) as us

union all 

select 
    cav.*,
    null as user_id, 
    null as my_vote,
    null as subscribed,
    null as saved
from comment_aggregates_fast cav;

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

-- user mention
create view user_mention_view as
select 
    c.id,
    um.id as user_mention_id,
    c.creator_id,
    c.creator_actor_id,
    c.creator_local,
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.community_id,
    c.community_actor_id,
    c.community_local,
    c.community_name,
    c.banned,
    c.banned_from_community,
    c.creator_name,
    c.creator_avatar,
    c.score,
    c.upvotes,
    c.downvotes,
    c.hot_rank,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id,
    (select actor_id from user_ u where u.id = um.recipient_id) as recipient_actor_id,
    (select local from user_ u where u.id = um.recipient_id) as recipient_local
from user_mention um, comment_view c
where um.comment_id = c.id;

create view user_mention_fast_view as 
select
    ac.id,
    um.id as user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    u.id as user_id,
    coalesce(cl.score, 0) as my_vote,
    (select cs.id::bool from comment_saved cs where u.id = cs.user_id and cs.comment_id = ac.id) as saved,
    um.recipient_id,
    (select actor_id from user_ u where u.id = um.recipient_id) as recipient_actor_id,
    (select local from user_ u where u.id = um.recipient_id) as recipient_local
from user_ u
cross join (
  select
  ca.*
  from comment_aggregates_fast ca
) ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id
left join user_mention um on um.comment_id = ac.id

union all

select 
    ac.id,
    um.id as user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.community_actor_id,
    ac.community_local,
    ac.community_name,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    null as user_id, 
    null as my_vote,
    null as saved,
    um.recipient_id,
    (select actor_id from user_ u where u.id = um.recipient_id) as recipient_actor_id,
    (select local from user_ u where u.id = um.recipient_id) as recipient_local
from comment_aggregates_fast ac
left join user_mention um on um.comment_id = ac.id
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

    -- Update community number of comments
    update community_aggregates_fast as caf
    set number_of_comments = number_of_comments - 1
    from post as p
    where caf.id = p.community_id and p.id = OLD.post_id;

  ELSIF (TG_OP = 'UPDATE') THEN
    delete from comment_aggregates_fast where id = OLD.id;
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.id;
  ELSIF (TG_OP = 'INSERT') THEN
    insert into comment_aggregates_fast select * from comment_aggregates_view where id = NEW.id;

    -- Update user view due to comment count
    update user_fast 
    set number_of_comments = number_of_comments + 1
    where id = NEW.creator_id;
    
    -- Update post view due to comment count, new comment activity time, but only on new posts
    -- TODO this could be done more efficiently
    delete from post_aggregates_fast where id = NEW.post_id;
    insert into post_aggregates_fast select * from post_aggregates_view where id = NEW.post_id;

    -- Force the hot rank as zero on week-older posts
    update post_aggregates_fast as paf
    set hot_rank = 0
    where paf.id = NEW.post_id and (paf.published < ('now'::timestamp - '1 week'::interval));

    -- Update community number of comments
    update community_aggregates_fast as caf
    set number_of_comments = number_of_comments + 1 
    from post as p
    where caf.id = p.community_id and p.id = NEW.post_id;

  END IF;

  return null;
end $$;


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
    update post_aggregates_fast 
    set score = case 
      when (OLD.score = 1) then score - 1 
      else score + 1 end,
    upvotes = case 
      when (OLD.score = 1) then upvotes - 1 
      else upvotes end,
    downvotes = case 
      when (OLD.score = -1) then downvotes - 1 
      else downvotes end
    where id = OLD.post_id;

  ELSIF (TG_OP = 'INSERT') THEN
    update post_aggregates_fast 
    set score = case 
      when (NEW.score = 1) then score + 1 
      else score - 1 end,
    upvotes = case 
      when (NEW.score = 1) then upvotes + 1 
      else upvotes end,
    downvotes = case 
      when (NEW.score = -1) then downvotes + 1 
      else downvotes end
    where id = NEW.post_id;
  END IF;

  return null;
end $$;

drop trigger refresh_post_like on post_like;
create trigger refresh_post_like
after insert or delete
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
    update comment_aggregates_fast 
    set score = case 
      when (OLD.score = 1) then score - 1 
      else score + 1 end,
    upvotes = case 
      when (OLD.score = 1) then upvotes - 1 
      else upvotes end,
    downvotes = case 
      when (OLD.score = -1) then downvotes - 1 
      else downvotes end
    where id = OLD.comment_id;

  ELSIF (TG_OP = 'INSERT') THEN
    update comment_aggregates_fast 
    set score = case 
      when (NEW.score = 1) then score + 1 
      else score - 1 end,
    upvotes = case 
      when (NEW.score = 1) then upvotes + 1 
      else upvotes end,
    downvotes = case 
      when (NEW.score = -1) then downvotes + 1 
      else downvotes end
    where id = NEW.comment_id;
  END IF;

  return null;
end $$;

drop trigger refresh_comment_like on comment_like;
create trigger refresh_comment_like
after insert or delete
on comment_like
for each row
execute procedure refresh_comment_like();

-- Community user ban

drop trigger refresh_community_user_ban on community_user_ban;
create trigger refresh_community_user_ban
after insert or delete -- Note this is missing after update
on community_user_ban
for each row
execute procedure refresh_community_user_ban();

-- select creator_name, banned_from_community from comment_fast_view where user_id = 4 and content = 'test_before_ban';
-- select creator_name, banned_from_community, community_id from comment_aggregates_fast where content = 'test_before_ban';
-- Sample insert 
-- insert into comment(creator_id, post_id, content) values (1198, 341, 'test_before_ban');
-- insert into community_user_ban(community_id, user_id) values (2, 1198);
-- Sample delete
-- delete from community_user_ban where user_id = 1198 and community_id = 2;
-- delete from comment where content = 'test_before_ban';
-- update comment_aggregates_fast set banned_from_community = false where creator_id = 1198 and community_id = 2;
create or replace function refresh_community_user_ban()
returns trigger language plpgsql
as $$
begin
  -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
  IF (TG_OP = 'DELETE') THEN
    update comment_aggregates_fast set banned_from_community = false where creator_id = OLD.user_id and community_id = OLD.community_id;
    update post_aggregates_fast set banned_from_community = false where creator_id = OLD.user_id and community_id = OLD.community_id;
  ELSIF (TG_OP = 'INSERT') THEN
    update comment_aggregates_fast set banned_from_community = true where creator_id = NEW.user_id and community_id = NEW.community_id;
    update post_aggregates_fast set banned_from_community = true where creator_id = NEW.user_id and community_id = NEW.community_id;
  END IF;

  return null;
end $$;

-- Community follower

drop trigger refresh_community_follower on community_follower;
create trigger refresh_community_follower
after insert or delete -- Note this is missing after update
on community_follower
for each row
execute procedure refresh_community_follower();

create or replace function refresh_community_follower()
returns trigger language plpgsql
as $$
begin
  IF (TG_OP = 'DELETE') THEN
    update community_aggregates_fast set number_of_subscribers = number_of_subscribers - 1 where id = OLD.community_id;
  ELSIF (TG_OP = 'INSERT') THEN
    update community_aggregates_fast set number_of_subscribers = number_of_subscribers + 1 where id = NEW.community_id;
  END IF;

  return null;
end $$;
