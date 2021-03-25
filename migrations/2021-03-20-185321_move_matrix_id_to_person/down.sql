alter table local_user add column matrix_user_id text;
alter table local_user add column admin boolean default false not null;

update local_user lu
set 
  matrix_user_id = p.matrix_user_id,
  admin = p.admin
from person p
where p.id = lu.person_id;

drop view person_alias_1, person_alias_2;
alter table person drop column matrix_user_id;
alter table person drop column admin;

-- Regenerate the person_alias views
create view person_alias_1 as select * from person;
create view person_alias_2 as select * from person;
