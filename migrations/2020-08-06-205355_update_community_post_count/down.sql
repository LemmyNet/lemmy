-- Drop first
drop view community_view;
drop view community_aggregates_view;
drop view community_fast_view;
drop table community_aggregates_fast;

create view community_aggregates_view as
select
    c.id,
    c.name,
    c.title,
    c.icon,
    c.banner,
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
    u.actor_id as creator_actor_id,
    u.local as creator_local,
    u.name as creator_name,
    u.preferred_username as creator_preferred_username,
    u.avatar as creator_avatar,
    cat.name as category_name,
    coalesce(cf.subs, 0) as number_of_subscribers,
    coalesce(cd.posts, 0) as number_of_posts,
    coalesce(cd.comments, 0) as number_of_comments,
    hot_rank(cf.subs, c.published) as hot_rank
from community c
left join user_ u on c.creator_id = u.id
left join category cat on c.category_id = cat.id
left join (
    select
        p.community_id,
        count(distinct p.id) as posts,
        count(distinct ct.id) as comments
    from post p
    join comment ct on p.id = ct.post_id
    group by p.community_id
) cd on cd.community_id = c.id
left join (
    select
        community_id,
        count(*) as subs
    from community_follower
    group by community_id
) cf on cf.community_id = c.id;

create view community_view as
select
    cv.*,
    us.user as user_id,
    us.is_subbed::bool as subscribed
from community_aggregates_view cv
cross join lateral (
	select
		u.id as user,
		coalesce(cf.community_id, 0) as is_subbed
	from user_ u
	left join community_follower cf on u.id = cf.user_id and cf.community_id = cv.id
) as us

union all

select
    cv.*,
    null as user_id,
    null as subscribed
from community_aggregates_view cv;

-- The community fast table

create table community_aggregates_fast as select * from community_aggregates_view;
alter table community_aggregates_fast add primary key (id);

create view community_fast_view as
select
ac.*,
u.id as user_id,
(select cf.id::boolean from community_follower cf where u.id = cf.user_id and ac.id = cf.community_id) as subscribed
from user_ u
cross join (
  select
  ca.*
  from community_aggregates_fast ca
) ac

union all

select
caf.*,
null as user_id,
null as subscribed
from community_aggregates_fast caf;