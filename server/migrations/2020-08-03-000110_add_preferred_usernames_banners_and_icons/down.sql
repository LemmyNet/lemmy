-- Drops first
drop view site_view;
drop table user_fast;
drop view user_view;
drop view post_fast_view;
drop table post_aggregates_fast;
drop view post_view;
drop view post_aggregates_view;
drop view community_moderator_view;
drop view community_follower_view;
drop view community_user_ban_view;
drop view community_view;
drop view community_aggregates_view;
drop view community_fast_view;
drop table community_aggregates_fast;
drop view private_message_view;
drop view user_mention_view;
drop view reply_fast_view;
drop view comment_fast_view;
drop view comment_view;
drop view user_mention_fast_view;
drop table comment_aggregates_fast;
drop view comment_aggregates_view;

alter table site 
  drop column icon,
  drop column banner;

alter table community 
  drop column icon,
  drop column banner;

alter table user_ drop column banner;

-- Site
create view site_view as 
select *,
(select name from user_ u where s.creator_id = u.id) as creator_name,
(select avatar from user_ u where s.creator_id = u.id) as creator_avatar,
(select count(*) from user_) as number_of_users,
(select count(*) from post) as number_of_posts,
(select count(*) from comment) as number_of_comments,
(select count(*) from community) as number_of_communities
from site s;

-- User
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

-- Post fast

create view post_aggregates_view as
select
	p.*,
	-- creator details
	u.actor_id as creator_actor_id,
	u."local" as creator_local,
	u."name" as creator_name,
  u.published as creator_published,
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

create table post_aggregates_fast as select * from post_aggregates_view;
alter table post_aggregates_fast add primary key (id);

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

-- Community
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


-- Private message
create view private_message_view as 
select        
pm.*,
u.name as creator_name,
u.avatar as creator_avatar,
u.actor_id as creator_actor_id,
u.local as creator_local,
u2.name as recipient_name,
u2.avatar as recipient_avatar,
u2.actor_id as recipient_actor_id,
u2.local as recipient_local
from private_message pm
inner join user_ u on u.id = pm.creator_id
inner join user_ u2 on u2.id = pm.recipient_id;


-- Comments, mentions, replies

create view comment_aggregates_view as
select
	ct.*,
	-- post details
	p."name" as post_name,
	p.community_id,
	-- community details
	c.actor_id as community_actor_id,
	c."local" as community_local,
	c."name" as community_name,
	-- creator details
	u.banned as banned,
  coalesce(cb.id, 0)::bool as banned_from_community,
	u.actor_id as creator_actor_id,
	u.local as creator_local,
	u.name as creator_name,
  u.published as creator_published,
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

create view user_mention_view as
select
    c.id,
    um.id as user_mention_id,
    c.creator_id,
    c.creator_actor_id,
    c.creator_local,
    c.post_id,
    c.post_name,
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
    ac.post_name,
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
    ac.post_name,
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

-- redoing the triggers
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
