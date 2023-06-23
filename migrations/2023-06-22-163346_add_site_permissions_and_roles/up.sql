-- create site_role table listing all permissions
create table site_role (
    id serial primary key not null,
    name text not null,
    -- roles management
    configure_site_roles boolean not null,
    assign_user_roles boolean not null,
    -- site management
    update_site_details boolean not null,
    -- community management
    hide_community boolean not null,
    transfer_community boolean not null,
    feature_post boolean not null,
    create_community boolean not null,
    remove_community boolean not null,
    modify_community boolean not null,
    view_removed_content boolean not null,
    distinguish_comment boolean not null,
    remove_comment boolean not null,
    remove_post boolean not null,
    lock_unlock_post boolean not null,
    manage_community_mods boolean not null,
    -- people management
    ban_person boolean not null,
    view_banned_persons boolean not null,
    -- report management
    view_private_message_reports boolean not null,
    resolve_private_message_reports boolean not null,
    view_post_reports boolean not null,
    resolve_post_reports boolean not null,
    view_comment_reports boolean not null,
    resolve_comment_reports boolean not null,
    -- registrations
    approve_registration boolean not null,
    view_registration boolean not null,
    -- purge content
    purge_comment boolean not null,
    purge_community boolean not null,
    purge_person boolean not null,
    purge_post boolean not null,
    -- miscellaneous
    view_modlog_names boolean not null,
    modify_custom_emoji boolean not null,
    unblockable boolean not null
);

-- insert the previously existing two roles (admin or regular user)
insert into site_role (name, hide_community, transfer_community, configure_site_roles, assign_user_roles, ban_person,
 view_banned_persons, feature_post, view_private_message_reports, resolve_private_message_reports, view_modlog_names,
 approve_registration, view_registration, create_community, view_removed_content, remove_community,
 modify_custom_emoji, update_site_details, purge_comment, purge_community, purge_person, purge_post, view_comment_reports,
 unblockable, view_post_reports, resolve_post_reports, resolve_comment_reports, distinguish_comment, lock_unlock_post,
 modify_community, remove_comment, remove_post, manage_community_mods)
values 
 ('admin', true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true, true),
 ('user', false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false);

-- add the site_role_id column to person, with default value as 'user' id
alter table person add column site_role_id int not null references site_role(id) default 2;

-- update existing users to get the correct role
update person set site_role_id = case when admin then 1 else 2 end;

-- drop unused admin column
alter table person drop column admin;

-- add default_site_role_id column to local_site with default value as 'user' id
alter table local_site 
    add column top_admin_role_id int not null references site_role(id) default 1,
    add column default_site_role_id int not null references site_role(id) default 2; 