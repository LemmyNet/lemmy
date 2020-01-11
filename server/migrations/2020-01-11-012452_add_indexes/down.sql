-- This file should undo anything in `up.sql`

drop index idx_post_creator;
drop index idx_post_community;

drop index idx_post_like_post;
drop index idx_post_like_user;

drop index idx_comment_creator;
drop index idx_comment_parent;
drop index idx_comment_post;

drop index idx_comment_like_comment;
drop index idx_comment_like_user;
drop index idx_comment_like_post;

drop index idx_community_creator;
drop index idx_community_category;

drop index idx_community_follower_community;
drop index idx_community_follower_user;

drop index idx_community_user_ban_community;
drop index idx_community_user_ban_user;

drop view post_view;
create view post_view as
with all_post as
(
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
  group by p.id
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


