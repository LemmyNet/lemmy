-- Add unique ap_id for private_message, comment, and post

-- Need to delete the possible dupes for ones that don't start with the fake one
delete from private_message a using (
  select min(id) as id, ap_id
    from private_message 
    group by ap_id having count(*) > 1
) b
where a.ap_id = b.ap_id 
and a.id <> b.id;

delete from post a using (
  select min(id) as id, ap_id
    from post 
    group by ap_id having count(*) > 1
) b
where a.ap_id = b.ap_id 
and a.id <> b.id;

delete from comment a using (
  select min(id) as id, ap_id
    from comment 
    group by ap_id having count(*) > 1
) b
where a.ap_id = b.ap_id 
and a.id <> b.id;

-- Replacing the current default on the columns, to the unique one
update private_message 
set ap_id = generate_unique_changeme()
where ap_id = 'http://fake.com';

update post 
set ap_id = generate_unique_changeme()
where ap_id = 'http://fake.com';

update comment 
set ap_id = generate_unique_changeme()
where ap_id = 'http://fake.com';

-- Add the unique indexes
alter table private_message alter column ap_id set not null;
alter table private_message alter column ap_id set default generate_unique_changeme();

alter table post alter column ap_id set not null;
alter table post alter column ap_id set default generate_unique_changeme();

alter table comment alter column ap_id set not null;
alter table comment alter column ap_id set default generate_unique_changeme();

-- Add the uniques, for user_ and community too
alter table private_message add constraint idx_private_message_ap_id unique (ap_id);
alter table post add constraint idx_post_ap_id unique (ap_id);
alter table comment add constraint idx_comment_ap_id unique (ap_id);
alter table user_ add constraint idx_user_actor_id unique (actor_id);
alter table community add constraint idx_community_actor_id unique (actor_id);
