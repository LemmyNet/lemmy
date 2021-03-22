alter table person add column matrix_user_id text;
alter table person add column admin boolean default false not null;

update person p
set 
  matrix_user_id = lu.matrix_user_id,
  admin = lu.admin
from local_user lu
where p.id = lu.person_id;

alter table local_user drop column matrix_user_id;
alter table local_user drop column admin;

-- Regenerate the person_alias views
drop view person_alias_1, person_alias_2;
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;
