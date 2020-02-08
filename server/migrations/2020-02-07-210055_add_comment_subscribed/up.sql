
-- Adding community name, hot_rank, to comment_view, user_mention_view, and subscribed to comment_view

-- Rebuild the comment view
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
(select co.name from post p, community co where p.id = c.post_id and p.community_id = co.id) as community_name,
(select u.banned from user_ u where c.creator_id = u.id) as banned,
(select cb.id::bool from community_user_ban cb, post p where c.creator_id = cb.user_id and p.id = c.post_id and p.community_id = cb.community_id) as banned_from_community,
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
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.community_id,
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
    um.recipient_id
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
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
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
    um.recipient_id
from user_ u
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id
left join user_mention um on um.comment_id = ac.id

union all

select 
    ac.id,
    um.id as user_mention_id,
    ac.creator_id,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
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
    um.recipient_id
from all_comment ac
left join user_mention um on um.comment_id = ac.id
;

