-- Adding a field to email admins for new reports
alter table local_site add column reports_email_admins boolean not null default false;
