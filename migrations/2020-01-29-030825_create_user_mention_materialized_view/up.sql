create view user_mention_mview as 
with all_comment as
(
  select
  ca.*
  from comment_aggregates_mview ca
)

select
    ac.id,
    um.id as user_mention_id,
    ac.creator_id,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    u.id as user_id,
    coalesce(cl.score, 0) as my_vote,
    (select cs.id::bool from comment_saved cs where u.id = cs.user_id and cs.comment_id = ac.id) as saved,
    um.recipient_id
from user_ u
cross join all_comment ac
left join comment_like cl on u.id = cl.user_id and ac.id = cl.comment_id
left join user_mention um on um.comment_id = ac.id

union all

select 
    ac.id,
    um.id as user_mention_id,
    ac.creator_id,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.community_id,
    ac.banned,
    ac.banned_from_community,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    null as user_id, 
    null as my_vote,
    null as saved,
    um.recipient_id
from all_comment ac
left join user_mention um on um.comment_id = ac.id
;

