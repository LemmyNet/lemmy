create view community_view as 
with all_community as
(
  select *,
  (select name from user_ u where c.creator_id = u.id) as creator_name,
  (select name from category ct where c.category_id = ct.id) as category_name,
  (select count(*) from community_follower cf where cf.community_id = c.id) as number_of_subscribers,
  (select count(*) from post p where p.community_id = c.id) as number_of_posts,
  (select count(*) from comment co, post p where c.id = p.community_id and p.id = co.post_id) as number_of_comments
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

create view community_moderator_view as 
select *,
(select name from user_ u where cm.user_id = u.id) as user_name,
(select name from community c where cm.community_id = c.id) as community_name
from community_moderator cm;

create view community_follower_view as 
select *,
(select name from user_ u where cf.user_id = u.id) as user_name,
(select name from community c where cf.community_id = c.id) as community_name
from community_follower cf;

create view community_user_ban_view as 
select *,
(select name from user_ u where cm.user_id = u.id) as user_name,
(select name from community c where cm.community_id = c.id) as community_name
from community_user_ban cm;

create view site_view as 
select *,
(select name from user_ u where s.creator_id = u.id) as creator_name,
(select count(*) from user_) as number_of_users,
(select count(*) from post) as number_of_posts,
(select count(*) from comment) as number_of_comments
from site s;
