drop view aliases::person_1, aliases::person_2;

alter table person drop column ban_expires;
alter table community_person_ban drop column expires;

create view aliases::person_1 as select * from person;
create view aliases::person_2 as select * from person;
