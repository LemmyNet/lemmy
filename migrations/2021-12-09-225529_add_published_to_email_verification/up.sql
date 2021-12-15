alter table email_verification add column published timestamp not null default now();
