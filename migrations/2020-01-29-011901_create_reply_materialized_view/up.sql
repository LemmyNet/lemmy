-- https://github.com/dessalines/lemmy/issues/197
drop view reply_view;

-- Do the reply_view referencing the comment_mview
create view reply_view as 
with closereply as (
    select 
    c2.id, 
    c2.creator_id as sender_id, 
    c.creator_id as recipient_id
    from comment c
    inner join comment c2 on c.id = c2.parent_id
    where c2.creator_id != c.creator_id
    -- Do union where post is null
    union
    select
    c.id,
    c.creator_id as sender_id,
    p.creator_id as recipient_id
    from comment c, post p
    where c.post_id = p.id and c.parent_id is null and c.creator_id != p.creator_id
)
select cv.*,
closereply.recipient_id
from comment_mview cv, closereply
where closereply.id = cv.id
;
