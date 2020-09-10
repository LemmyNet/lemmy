-- Drop the triggers
drop trigger refresh_private_message on private_message;
drop function refresh_private_message();

-- Drop the view and table
drop view private_message_view cascade;
drop table private_message;

-- Rebuild the old views
drop view user_view cascade;
create view user_view as 
select 
u.id,
u.name,
u.avatar,
u.email,
u.fedi_name,
u.admin,
u.banned,
u.show_avatars,
u.send_notifications_to_email,
u.published,
(select count(*) from post p where p.creator_id = u.id) as number_of_posts,
(select coalesce(sum(score), 0) from post p, post_like pl where u.id = p.creator_id and p.id = pl.post_id) as post_score,
(select count(*) from comment c where c.creator_id = u.id) as number_of_comments,
(select coalesce(sum(score), 0) from comment c, comment_like cl where u.id = c.creator_id and c.id = cl.comment_id) as comment_score
from user_ u;

create materialized view user_mview as select * from user_view;

create unique index idx_user_mview_id on user_mview (id);

-- Drop the columns
alter table user_ drop column matrix_user_id;
