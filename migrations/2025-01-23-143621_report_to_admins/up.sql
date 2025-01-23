alter table post_report add column to_local_admins bool not null default false;
alter table comment_report add column to_local_admins bool not null default false;
alter table community_report add column to_local_admins bool not null default false;