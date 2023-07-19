drop view person_alias_1, person_alias_2;
alter table person drop column bot_account;
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;

alter table local_user drop column show_bot_accounts;
