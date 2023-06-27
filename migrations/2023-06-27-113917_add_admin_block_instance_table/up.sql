-- add the admin_block_instance table

create table admin_block_instance (
    id serial primary key,
    admin_person_id int references person on update cascade on delete cascade not null,
    instance_id int references instance on update cascade on delete cascade not null,
    reason text,
    blocked boolean,
    when_ timestamp not null default now()
);