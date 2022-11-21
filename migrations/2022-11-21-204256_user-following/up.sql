-- create user follower table with two references to persons
create table person_follower (
    id serial primary key,
    person_id int not null,
    follower_id int not null,
    published timestamp not null default now(),
    pending boolean not null default false,
    unique (follower_id, person_id)
);

alter table person_follower add foreign key (follower_id) references person on update cascade on delete cascade;
alter table person_follower add foreign key (person_id) references person on update cascade on delete cascade;

alter table community_follower alter column pending set not null;
