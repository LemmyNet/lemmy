-- Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity
create or replace function hot_rank(
  score numeric,
  published timestamp without time zone)
returns numeric as $$
begin
  -- hours_diff:=EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600
  return 10000*sign(score)*log(1 + abs(score)) / power(((EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600) + 2), 1.8);
end; $$
LANGUAGE plpgsql;

create view post_listing as 
select post.*, 
(select count(*) from comment where comment.post_id = post.id) as number_of_comments,
coalesce(sum(post_like.score),0) as score,
hot_rank(coalesce(sum(post_like.score),0), post.published) as hot_rank
from post
left join post_like
on post.id = post_like.post_id
group by post.id;
