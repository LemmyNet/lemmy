DROP VIEW person_alias_1;
DROP VIEW person_alias_2;

ALTER TABLE community ALTER COLUMN name TYPE varchar(255);
ALTER TABLE community ALTER COLUMN title TYPE varchar(255);
ALTER TABLE person ALTER COLUMN name TYPE varchar(255);
ALTER TABLE person ALTER COLUMN display_name TYPE varchar(255);

create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;
