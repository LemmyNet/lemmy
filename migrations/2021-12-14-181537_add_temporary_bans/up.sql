-- Add ban_expires to person, community_person_ban
alter table person add column ban_expires timestamp;
alter table community_person_ban add column expires timestamp;

drop view aliases::person_1, aliases::person_2;
create view aliases::person_1 as select * from person;
create view aliases::person_2 as select * from person;

