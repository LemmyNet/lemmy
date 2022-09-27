-- Adding a field to email admins for new applications
alter table site add column application_email_admins boolean not null default false;
