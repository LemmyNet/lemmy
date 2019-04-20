create view comment_view as
with all_comment as
(
  select        
  c.*,
  (select community_id from post p where p.id = c.post_id),
  (select u.banned from user_ u where c.creator_id = u.id) as banned,
  (select cb.id::bool from community_user_ban cb, post p where c.creator_id = cb.user_id and p.id = c.post_id and p.community_id = cb.community_id) as banned_from_community,
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
(select cs.id::bool from comment_saved cs where u.id = cs.user_id and cs.comment_id = ac.id) as saved
from user_ u
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id

union all

select 
    ac.*,
    null as user_id, 
    null as my_vote,
    null as saved
from all_comment ac
;
