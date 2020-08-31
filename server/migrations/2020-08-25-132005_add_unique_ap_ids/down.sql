-- Drop the uniques
alter table private_message drop constraint idx_private_message_ap_id;
alter table post drop constraint idx_post_ap_id;
alter table comment drop constraint idx_comment_ap_id;
alter table user_ drop constraint idx_user_actor_id;
alter table community drop constraint idx_community_actor_id;

alter table private_message alter column ap_id set not null;
alter table private_message alter column ap_id set default 'http://fake.com';

alter table post alter column ap_id set not null;
alter table post alter column ap_id set default 'http://fake.com';

alter table comment alter column ap_id set not null;
alter table comment alter column ap_id set default 'http://fake.com';

update private_message
set ap_id = 'http://fake.com'
where ap_id like 'changeme_%';

update post
set ap_id = 'http://fake.com'
where ap_id like 'changeme_%';

update comment
set ap_id = 'http://fake.com'
where ap_id like 'changeme_%';
