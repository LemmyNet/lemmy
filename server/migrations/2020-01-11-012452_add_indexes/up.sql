-- Go through all the tables joins, optimize every view, CTE, etc.

create index idx_post_creator on post (creator_id);
create index idx_post_community on post (community_id);

create index idx_post_like_post on post_like (post_id);
create index idx_post_like_user on post_like (user_id);

create index idx_comment_creator on comment (creator_id);
create index idx_comment_parent on comment (parent_id);
create index idx_comment_post on comment (post_id);

create index idx_comment_like_comment on comment_like (comment_id);
create index idx_comment_like_user on comment_like (user_id);
create index idx_comment_like_post on comment_like (post_id);

create index idx_community_creator on community (creator_id);
create index idx_community_category on community (category_id);

create index idx_community_follower_community on community_follower (community_id);
create index idx_community_follower_user on community_follower (user_id);

create index idx_community_user_ban_community on community_user_ban (community_id);
create index idx_community_user_ban_user on community_user_ban (user_id);

-- optimize post_view

drop view post_view;
create view post_view as
with all_post as
(
  select        
  p.*,
  u.banned as banned,
  (select cb.id::bool from community_user_ban cb where p.creator_id = cb.user_id and p.community_id = cb.community_id) as banned_from_community,
  u.name as creator_name, 
  u.avatar as creator_avatar,
  c.name as community_name, 
  c.removed as community_removed,
  c.deleted as community_deleted,
  c.nsfw as community_nsfw,
  (select count(*) from comment where comment.post_id = p.id) as number_of_comments,
  coalesce(sum(pl.score), 0) as score,
  count (case when pl.score = 1 then 1 else null end) as upvotes,
  count (case when pl.score = -1 then 1 else null end) as downvotes,
  hot_rank(coalesce(sum(pl.score) , 0), p.published) as hot_rank
  from post p
  left join post_like pl on p.id = pl.post_id
  inner join user_ u on p.creator_id = u.id
  inner join community c on p.community_id = c.id
  group by p.id, u.banned, u.name, u.avatar, c.name, c.removed, c.deleted, c.nsfw
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
