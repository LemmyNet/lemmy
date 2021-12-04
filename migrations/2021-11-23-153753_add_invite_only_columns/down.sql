-- Add columns to site table
alter table site drop column require_application;
alter table site drop column application_question;

-- Add pending to local_user
alter table local_user drop column accepted_application;

drop table registration_application;
