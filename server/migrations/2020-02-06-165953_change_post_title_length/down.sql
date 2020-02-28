-- Drop the dependent views
drop view post_view;
drop view post_mview;
drop materialized view post_aggregates_mview;
drop view post_aggregates_view;
drop view mod_remove_post_view;
drop view mod_sticky_post_view;
drop view mod_lock_post_view;
drop view mod_remove_comment_view;

alter table post alter column name type varchar(100);

-- regen post view
create view post_aggregates_view as
select        
p.*,
(select u.banned from user_ u where p.creator_id = u.id) as banned,
(select cb.id::bool from community_user_ban cb where p.creator_id = cb.user_id and p.community_id = cb.community_id) as banned_from_community,
(select name from user_ where p.creator_id = user_.id) as creator_name,
(select avatar from user_ where p.creator_id = user_.id) as creator_avatar,
(select name from community where p.community_id = community.id) as community_name,
(select removed from community c where p.community_id = c.id) as community_removed,
(select deleted from community c where p.community_id = c.id) as community_deleted,
(select nsfw from community c where p.community_id = c.id) as community_nsfw,
(select count(*) from comment where comment.post_id = p.id) as number_of_comments,
coalesce(sum(pl.score), 0) as score,
count (case when pl.score = 1 then 1 else null end) as upvotes,
count (case when pl.score = -1 then 1 else null end) as downvotes,
hot_rank(coalesce(sum(pl.score) , 0), p.published) as hot_rank
from post p
left join post_like pl on p.id = pl.post_id
group by p.id;

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

-- The mod views

create view mod_remove_post_view as 
select mrp.*,
(select name from user_ u where mrp.mod_user_id = u.id) as mod_user_name,
(select name from post p where mrp.post_id = p.id) as post_name,
(select c.id from post p, community c where mrp.post_id = p.id and p.community_id = c.id) as community_id,
(select c.name from post p, community c where mrp.post_id = p.id and p.community_id = c.id) as community_name
from mod_remove_post mrp;

create view mod_lock_post_view as 
select mlp.*,
(select name from user_ u where mlp.mod_user_id = u.id) as mod_user_name,
(select name from post p where mlp.post_id = p.id) as post_name,
(select c.id from post p, community c where mlp.post_id = p.id and p.community_id = c.id) as community_id,
(select c.name from post p, community c where mlp.post_id = p.id and p.community_id = c.id) as community_name
from mod_lock_post mlp;

create view mod_remove_comment_view as 
select mrc.*,
(select name from user_ u where mrc.mod_user_id = u.id) as mod_user_name,
(select c.id from comment c where mrc.comment_id = c.id) as comment_user_id,
(select name from user_ u, comment c where mrc.comment_id = c.id and u.id = c.creator_id) as comment_user_name,
(select content from comment c where mrc.comment_id = c.id) as comment_content,
(select p.id from post p, comment c where mrc.comment_id = c.id and c.post_id = p.id) as post_id,
(select p.name from post p, comment c where mrc.comment_id = c.id and c.post_id = p.id) as post_name,
(select co.id from comment c, post p, community co where mrc.comment_id = c.id and c.post_id = p.id and p.community_id = co.id) as community_id, 
(select co.name from comment c, post p, community co where mrc.comment_id = c.id and c.post_id = p.id and p.community_id = co.id) as community_name
from mod_remove_comment mrc;

create view mod_sticky_post_view as 
select msp.*,
(select name from user_ u where msp.mod_user_id = u.id) as mod_user_name,
(select name from post p where msp.post_id = p.id) as post_name,
(select c.id from post p, community c where msp.post_id = p.id and p.community_id = c.id) as community_id,
(select c.name from post p, community c where msp.post_id = p.id and p.community_id = c.id) as community_name
from mod_sticky_post msp;
