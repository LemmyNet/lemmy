-- re-add admin column
alter table person add column admin boolean default false not null,
    drop column site_role_id;

-- we can't assume roles haven't changed, so if we downgrade the database we'll just restore the first user as admin
-- todo: this seems dodgy
update person 
    set admin = true
    where id = 1;
alter table local_site drop column default_site_role_id, drop column top_admin_role_id;

drop table site_role;
