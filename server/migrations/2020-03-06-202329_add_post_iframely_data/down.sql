-- Adds a newest_activity_time for the post_views, in order to sort by newest comment
drop view post_view;
drop view post_mview;
drop materialized view post_aggregates_mview;
drop view post_aggregates_view;

-- Drop the columns
alter table post drop column embed_title;
alter table post drop column embed_description;
alter table post drop column embed_html;
alter table post drop column thumbnail_url;

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

