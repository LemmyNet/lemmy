-- Add ban_expires to person, community_person_ban
alter table person add column ban_expires timestamp;
alter table community_person_ban add column expires timestamp;

drop view person_alias_1, person_alias_2;
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;

