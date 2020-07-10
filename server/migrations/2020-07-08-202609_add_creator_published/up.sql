drop view user_mention_view;
drop view reply_fast_view;
drop view comment_fast_view;
drop view comment_view;

drop view user_mention_fast_view;
drop table comment_aggregates_fast;
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

-- add creator_published to the post view
drop view post_fast_view;
drop table post_aggregates_fast;
drop view post_view;
drop view post_aggregates_view;

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