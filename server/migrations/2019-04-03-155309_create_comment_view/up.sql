create view comment_view as
with all_comment as
(
  select        
  c.*,
  (select community_id from post p where p.id = c.post_id),
  (select cb.id::bool from community_user_ban cb where c.creator_id = cb.user_id) as banned,
  (select name from user_ where c.creator_id = user_.id) as creator_name,
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
coalesce(cl.score, 0) as my_vote,
u.admin or (select cm.id::bool from community_moderator cm, post p where u.id = cm.user_id and ac.post_id = p.id and p.community_id = cm.community_id) as am_mod
from user_ u
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id

union all

select 
    ac.*,
    null as user_id, 
    null as my_vote,
    null as am_mod
from all_comment ac
;
