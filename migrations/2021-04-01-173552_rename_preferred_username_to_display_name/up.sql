alter table person rename preferred_username to display_name;

-- Regenerate the person_alias views
drop view aliases::person_1, aliases::person_2;
create view aliases::person_1 as select * from person;
create view aliases::person_2 as select * from person;
