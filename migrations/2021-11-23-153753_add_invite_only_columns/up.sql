-- Add columns to site table
alter table site add column require_application boolean not null default false;
alter table site add column require_email boolean not null default false;
alter table site add column application_question text;

-- Add pending to local_user
alter table local_user add column accepted_application boolean not null default false;
alter table local_user add column verified_email boolean not null default false;

create table registration_application (
  id serial primary key,
  local_user_id int references local_user on update cascade on delete cascade not null,
  answer text not null,
  acceptor_id int references person on update cascade on delete cascade,
  accepted boolean not null default false,
  deny_reason text,
  published timestamp not null default now(),
  unique(local_user_id)
);
