-- Add columns
alter table user_ add column show_avatars boolean default true not null;
alter table user_ add column send_notifications_to_email boolean default false not null;

-- Rebuild the user_view
drop view user_view;
create view user_view as 
select id,
name,
avatar,
email,
fedi_name,
admin,
banned,
show_avatars,
send_notifications_to_email,
published,
(select count(*) from post p where p.creator_id = u.id) as number_of_posts,
(select coalesce(sum(score), 0) from post p, post_like pl where u.id = p.creator_id and p.id = pl.post_id) as post_score,
(select count(*) from comment c where c.creator_id = u.id) as number_of_comments,
(select coalesce(sum(score), 0) from comment c, comment_like cl where u.id = c.creator_id and c.id = cl.comment_id) as comment_score
from user_ u;
