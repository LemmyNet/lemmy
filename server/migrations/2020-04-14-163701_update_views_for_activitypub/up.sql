-- user_view
drop view user_view cascade;

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
(select count(*) from post p where p.creator_id = u.id) as number_of_posts,
(select coalesce(sum(score), 0) from post p, post_like pl where u.id = p.creator_id and p.id = pl.post_id) as post_score,
(select count(*) from comment c where c.creator_id = u.id) as number_of_comments,
(select coalesce(sum(score), 0) from comment c, comment_like cl where u.id = c.creator_id and c.id = cl.comment_id) as comment_score
from user_ u;

create materialized view user_mview as select * from user_view;

create unique index idx_user_mview_id on user_mview (id);

-- community_view
drop view community_aggregates_view cascade;
create view community_aggregates_view as
-- Now that there's public and private keys, you have to be explicit here
select c.id,
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
(select actor_id from user_ u where c.creator_id = u.id) as creator_actor_id,
(select local from user_ u where c.creator_id = u.id) as creator_local,
(select name from user_ u where c.creator_id = u.id) as creator_name,
(select avatar from user_ u where c.creator_id = u.id) as creator_avatar,
(select name from category ct where c.category_id = ct.id) as category_name,
(select count(*) from community_follower cf where cf.community_id = c.id) as number_of_subscribers,
(select count(*) from post p where p.community_id = c.id) as number_of_posts,
(select count(*) from comment co, post p where c.id = p.community_id and p.id = co.post_id) as number_of_comments,
hot_rank((select count(*) from community_follower cf where cf.community_id = c.id), c.published) as hot_rank
from community c;

create materialized view community_aggregates_mview as select * from community_aggregates_view;

create unique index idx_community_aggregates_mview_id on community_aggregates_mview (id);

create view community_view as
with all_community as
(
  select
  ca.*
  from community_aggregates_view ca
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
from all_community ac
;

create view community_mview as
with all_community as
(
  select
  ca.*
  from community_aggregates_mview ca
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
from all_community ac
;

-- community views
drop view community_moderator_view;
drop view community_follower_view;
drop view community_user_ban_view;

create view community_moderator_view as 
select *,
(select actor_id from user_ u where cm.user_id = u.id) as user_actor_id,
(select local from user_ u where cm.user_id = u.id) as user_local,
(select name from user_ u where cm.user_id = u.id) as user_name,
(select avatar from user_ u where cm.user_id = u.id),
(select actor_id from community c where cm.community_id = c.id) as community_actor_id,
(select local from community c where cm.community_id = c.id) as community_local,
(select name from community c where cm.community_id = c.id) as community_name
from community_moderator cm;

create view community_follower_view as 
select *,
(select actor_id from user_ u where cf.user_id = u.id) as user_actor_id,
(select local from user_ u where cf.user_id = u.id) as user_local,
(select name from user_ u where cf.user_id = u.id) as user_name,
(select avatar from user_ u where cf.user_id = u.id),
(select actor_id from community c where cf.community_id = c.id) as community_actor_id,
(select local from community c where cf.community_id = c.id) as community_local,
(select name from community c where cf.community_id = c.id) as community_name
from community_follower cf;

create view community_user_ban_view as 
select *,
(select actor_id from user_ u where cm.user_id = u.id) as user_actor_id,
(select local from user_ u where cm.user_id = u.id) as user_local,
(select name from user_ u where cm.user_id = u.id) as user_name,
(select avatar from user_ u where cm.user_id = u.id),
(select actor_id from community c where cm.community_id = c.id) as community_actor_id,
(select local from community c where cm.community_id = c.id) as community_local,
(select name from community c where cm.community_id = c.id) as community_name
from community_user_ban cm;

-- post_view
drop view post_view;
drop view post_mview;
drop materialized view post_aggregates_mview;
drop view post_aggregates_view;

-- regen post view
create view post_aggregates_view as
select        
p.*,
(select u.banned from user_ u where p.creator_id = u.id) as banned,
(select cb.id::bool from community_user_ban cb where p.creator_id = cb.user_id and p.community_id = cb.community_id) as banned_from_community,
(select actor_id from user_ where p.creator_id = user_.id) as creator_actor_id,
(select local from user_ where p.creator_id = user_.id) as creator_local,
(select name from user_ where p.creator_id = user_.id) as creator_name,
(select avatar from user_ where p.creator_id = user_.id) as creator_avatar,
(select actor_id from community where p.community_id = community.id) as community_actor_id,
(select local from community where p.community_id = community.id) as community_local,
(select name from community where p.community_id = community.id) as community_name,
(select removed from community c where p.community_id = c.id) as community_removed,
(select deleted from community c where p.community_id = c.id) as community_deleted,
(select nsfw from community c where p.community_id = c.id) as community_nsfw,
(select count(*) from comment where comment.post_id = p.id) as number_of_comments,
coalesce(sum(pl.score), 0) as score,
count (case when pl.score = 1 then 1 else null end) as upvotes,
count (case when pl.score = -1 then 1 else null end) as downvotes,
hot_rank(coalesce(sum(pl.score) , 0), 
  (
    case when (p.published < ('now'::timestamp - '1 month'::interval)) then p.published -- Prevents necro-bumps
    else greatest(c.recent_comment_time, p.published)
    end
  )
) as hot_rank,
(
  case when (p.published < ('now'::timestamp - '1 month'::interval)) then p.published -- Prevents necro-bumps
  else greatest(c.recent_comment_time, p.published)
  end
) as newest_activity_time
from post p
left join post_like pl on p.id = pl.post_id
left join (
  select post_id, 
  max(published) as recent_comment_time
  from comment
  group by 1
) c on p.id = c.post_id
group by p.id, c.recent_comment_time;

create materialized view post_aggregates_mview as select * from post_aggregates_view;

create unique index idx_post_aggregates_mview_id on post_aggregates_mview (id);

create view post_view as 
with all_post as (
  select
  pa.*
  from post_aggregates_view pa
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

create view post_mview as 
with all_post as (
  select
  pa.*
  from post_aggregates_mview pa
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


-- reply_view, comment_view, user_mention
drop view reply_view;
drop view user_mention_view;
drop view user_mention_mview;
drop view comment_view;
drop view comment_mview;
drop materialized view comment_aggregates_mview;
drop view comment_aggregates_view;

-- reply and comment view
create view comment_aggregates_view as
select        
c.*,
(select community_id from post p where p.id = c.post_id),
(select co.actor_id from post p, community co where p.id = c.post_id and p.community_id = co.id) as community_actor_id,
(select co.local from post p, community co where p.id = c.post_id and p.community_id = co.id) as community_local,
(select co.name from post p, community co where p.id = c.post_id and p.community_id = co.id) as community_name,
(select u.banned from user_ u where c.creator_id = u.id) as banned,
(select cb.id::bool from community_user_ban cb, post p where c.creator_id = cb.user_id and p.id = c.post_id and p.community_id = cb.community_id) as banned_from_community,
(select actor_id from user_ where c.creator_id = user_.id) as creator_actor_id,
(select local from user_ where c.creator_id = user_.id) as creator_local,
(select name from user_ where c.creator_id = user_.id) as creator_name,
(select avatar from user_ where c.creator_id = user_.id) as creator_avatar,
coalesce(sum(cl.score), 0) as score,
count (case when cl.score = 1 then 1 else null end) as upvotes,
count (case when cl.score = -1 then 1 else null end) as downvotes,
hot_rank(coalesce(sum(cl.score) , 0), c.published) as hot_rank
from comment c
left join comment_like cl on c.id = cl.comment_id
group by c.id;

create materialized view comment_aggregates_mview as select * from comment_aggregates_view;

create unique index idx_comment_aggregates_mview_id on comment_aggregates_mview (id);

create view comment_view as
with all_comment as
(
  select
  ca.*
  from comment_aggregates_view ca
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

create view comment_mview as
with all_comment as
(
  select
  ca.*
  from comment_aggregates_mview ca
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

-- Do the reply_view referencing the comment_mview
create view reply_view as 
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
from comment_mview cv, closereply
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


create view user_mention_mview as 
with all_comment as
(
  select
  ca.*
  from comment_aggregates_mview ca
)

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
cross join all_comment ac
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
from all_comment ac
left join user_mention um on um.comment_id = ac.id
;

