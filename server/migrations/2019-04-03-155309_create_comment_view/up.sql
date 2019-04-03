create view comment_view as
with all_comment as
(
  select        
  c.*,
  (select name from user_ where c.creator_id = user_.id) creator_name,
  coalesce(sum(cl.score), 0) as score,
  count (case when cl.score = 1 then 1 else null end) as upvotes,
  count (case when cl.score = -1 then 1 else null end) as downvotes
  from comment c
  left join comment_like cl on c.id = cl.comment_id
  group by c.id
)

select
ac.*,
u.id as user_id,
coalesce(cl.score, 0) as my_vote
from user_ u
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id

union all

select 
    ac.*,
    null as user_id, 
    null as my_vote
from all_comment ac
;
