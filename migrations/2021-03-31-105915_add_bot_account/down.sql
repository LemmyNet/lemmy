drop view aliases::person_1, aliases::person_2;
alter table person drop column bot_account;
create view aliases::person_1 as select * from person;
create view aliases::person_2 as select * from person;

alter table local_user drop column show_bot_accounts;
