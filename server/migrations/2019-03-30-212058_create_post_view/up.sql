-- Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity
create or replace function hot_rank(
  score numeric,
  published timestamp without time zone)
returns integer as $$
begin
  -- hours_diff:=EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600
  return 10000*sign(score)*log(1 + abs(score)) / power(((EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600) + 2), 1.8);
end; $$
LANGUAGE plpgsql;

create view post_view as
with all_post as
(
  select        
  p.*,
  (select name from user_ where p.creator_id = user_.id) creator_name,
  (select name from community where p.community_id = community.id) as community_name,
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
coalesce(pl.score, 0) as my_vote
from user_ u
cross join all_post ap
left join post_like pl on u.id = pl.user_id and ap.id = pl.post_id

union all

select 
ap.*,
null as user_id,
null as my_vote
from all_post ap
;

/* The old post view */
/* create view post_view as */
/* select */ 
/* u.id as user_id, */
/* pl.score as my_vote, */
/* p.id as id, */ 
/* p.name as name, */
/* p.url, */
/* p.body, */
/* p.creator_id, */
/* (select name from user_ where p.creator_id = user_.id) creator_name, */
/* p.community_id, */
/* (select name from community where p.community_id = community.id) as community_name, */
/* (select count(*) from comment where comment.post_id = p.id) as number_of_comments, */
/* coalesce(sum(pl.score) over (partition by p.id), 0) as score, */
/* count (case when pl.score = 1 then 1 else null end) over (partition by p.id) as upvotes, */
/* count (case when pl.score = -1 then 1 else null end) over (partition by p.id) as downvotes, */
/* hot_rank(coalesce(sum(pl.score) over (partition by p.id) , 0), p.published) as hot_rank, */
/* p.published, */
/* p.updated */
/* from user_ u */
/* cross join post p */
/* left join post_like pl on u.id = pl.user_id and p.id = pl.post_id; */



