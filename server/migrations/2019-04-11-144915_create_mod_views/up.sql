create view mod_remove_post_view as 
select mrp.*,
(select name from user_ u where mrp.mod_user_id = u.id) as mod_user_name,
(select name from post p where mrp.post_id = p.id) as post_name,
(select c.id from post p, community c where mrp.post_id = p.id and p.community_id = c.id) as community_id,
(select c.name from post p, community c where mrp.post_id = p.id and p.community_id = c.id) as community_name
from mod_remove_post mrp;

create view mod_lock_post_view as 
select mlp.*,
(select name from user_ u where mlp.mod_user_id = u.id) as mod_user_name,
(select name from post p where mlp.post_id = p.id) as post_name,
(select c.id from post p, community c where mlp.post_id = p.id and p.community_id = c.id) as community_id,
(select c.name from post p, community c where mlp.post_id = p.id and p.community_id = c.id) as community_name
from mod_lock_post mlp;

create view mod_remove_comment_view as 
select mrc.*,
(select name from user_ u where mrc.mod_user_id = u.id) as mod_user_name,
(select c.id from comment c where mrc.comment_id = c.id) as comment_user_id,
(select name from user_ u, comment c where mrc.comment_id = c.id and u.id = c.creator_id) as comment_user_name,
(select content from comment c where mrc.comment_id = c.id) as comment_content,
(select p.id from post p, comment c where mrc.comment_id = c.id and c.post_id = p.id) as post_id,
(select p.name from post p, comment c where mrc.comment_id = c.id and c.post_id = p.id) as post_name,
(select co.id from comment c, post p, community co where mrc.comment_id = c.id and c.post_id = p.id and p.community_id = co.id) as community_id, 
(select co.name from comment c, post p, community co where mrc.comment_id = c.id and c.post_id = p.id and p.community_id = co.id) as community_name
from mod_remove_comment mrc;

create view mod_remove_community_view as 
select mrc.*,
(select name from user_ u where mrc.mod_user_id = u.id) as mod_user_name,
(select c.name from community c where mrc.community_id = c.id) as community_name
from mod_remove_community mrc;

create view mod_ban_from_community_view as 
select mb.*,
(select name from user_ u where mb.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where mb.other_user_id = u.id) as other_user_name,
(select name from community c where mb.community_id = c.id) as community_name
from mod_ban_from_community mb;

create view mod_ban_view as 
select mb.*,
(select name from user_ u where mb.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where mb.other_user_id = u.id) as other_user_name
from mod_ban mb;

create view mod_add_community_view as 
select ma.*,
(select name from user_ u where ma.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where ma.other_user_id = u.id) as other_user_name,
(select name from community c where ma.community_id = c.id) as community_name
from mod_add_community ma;

create view mod_add_view as 
select ma.*,
(select name from user_ u where ma.mod_user_id = u.id) as mod_user_name,
(select name from user_ u where ma.other_user_id = u.id) as other_user_name
from mod_add ma;
