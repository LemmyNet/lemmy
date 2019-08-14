drop view community_view;
drop view post_view;
alter table community drop column nsfw;
alter table post drop column nsfw;
alter table user_ drop column show_nsfw;

-- the views
create view community_view as 
with all_community as
(
  select *,
  (select name from user_ u where c.creator_id = u.id) as creator_name,
  (select name from category ct where c.category_id = ct.id) as category_name,
  (select count(*) from community_follower cf where cf.community_id = c.id) as number_of_subscribers,
  (select count(*) from post p where p.community_id = c.id) as number_of_posts,
  (select count(*) from comment co, post p where c.id = p.community_id and p.id = co.post_id) as number_of_comments,
  hot_rank((select count(*) from community_follower cf where cf.community_id = c.id), c.published) as hot_rank
  from community c
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


-- Post view
create view post_view as
with all_post as
(
  select        
  p.*,
  (select name from user_ where p.creator_id = user_.id) as creator_name,
  (select name from community where p.community_id = community.id) as community_name,
  (select removed from community c where p.community_id = c.id) as community_removed,
  (select deleted from community c where p.community_id = c.id) as community_deleted,
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

