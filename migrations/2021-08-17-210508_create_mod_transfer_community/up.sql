-- Add the mod_transfer_community log table
create table mod_transfer_community (
  id serial primary key,
  mod_person_id int references person on update cascade on delete cascade not null,
  other_person_id int references person on update cascade on delete cascade not null,
  community_id int references community on update cascade on delete cascade not null,
  removed boolean default false,
  when_ timestamp not null default now()
);
