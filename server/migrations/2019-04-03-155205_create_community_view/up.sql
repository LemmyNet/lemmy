create view community_view as 
select *,
(select name from user_ u where c.creator_id = u.id) as creator_name,
(select name from category ct where c.category_id = ct.id) as category_name,
(select count(*) from community_follower cf where cf.community_id = c.id) as number_of_subscribers,
(select count(*) from post p where p.community_id = c.id) as number_of_posts
from community c;

create view community_moderator_view as 
select *,
(select name from user_ u where cm.user_id = u.id) as user_name
from community_moderator cm;

create view community_follower_view as 
select *,
(select name from user_ u where cf.user_id = u.id) as user_name
from community_follower cf;
