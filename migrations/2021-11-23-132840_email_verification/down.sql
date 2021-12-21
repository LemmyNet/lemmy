-- revert defaults from db for local user init
alter table local_user alter column theme set default 'darkly';
alter table local_user alter column default_listing_type set default 1;

-- remove tables and columns for optional email verification
alter table site drop column require_email_verification;
alter table local_user drop column email_verified;
drop table email_verification;
