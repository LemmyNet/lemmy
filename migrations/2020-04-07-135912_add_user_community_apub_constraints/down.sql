-- User table
drop view user_view cascade;

alter table user_ 
add column fedi_name varchar(40) not null default 'http://fake.com';

alter table user_
add constraint user__name_fedi_name_key unique (name, fedi_name);

-- Community
alter table community
add constraint community_name_key unique (name);


create view user_view as 
select 
u.id,
u.name,
u.avatar,
u.email,
u.matrix_user_id,
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
