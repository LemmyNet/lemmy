alter table community add column hidden boolean default false;


create table mod_hide_community
(
    id serial primary key,
    community_id int references community on update cascade on delete cascade not null,
    mod_person_id int references person on update cascade on delete cascade not null,
    when_ timestamp not null default now(),
    reason text,
    hidden boolean default false
);

