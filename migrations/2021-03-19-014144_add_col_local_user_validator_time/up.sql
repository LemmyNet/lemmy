alter table local_user add column validator_time timestamp not null default now();
