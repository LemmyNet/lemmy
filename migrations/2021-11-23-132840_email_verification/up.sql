-- use defaults from db for local user init
alter table local_user alter column theme set default 'browser';
alter table local_user alter column default_listing_type set default 2;

-- add tables and columns for optional email verification
alter table site add column require_email_verification boolean not null default false;
alter table local_user add column email_verified boolean not null default false;

create table email_verification (
    id serial primary key,
    local_user_id int references local_user(id) on update cascade on delete cascade not null,
    email text not null,
    verification_token text not null
);
