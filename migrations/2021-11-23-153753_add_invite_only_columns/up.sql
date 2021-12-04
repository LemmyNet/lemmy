-- Add columns to site table
alter table site add column require_application boolean not null default false;
alter table site add column application_question text;
alter table site add column private_instance boolean not null default false;

-- Add pending to local_user
alter table local_user add column accepted_application boolean not null default false;

create table registration_application (
  id serial primary key,
  local_user_id int references local_user on update cascade on delete cascade not null,
  answer text not null,
  admin_id int references person on update cascade on delete cascade,
  deny_reason text,
  published timestamp not null default now(),
  unique(local_user_id)
);

create index idx_registration_application_published on registration_application (published desc);
