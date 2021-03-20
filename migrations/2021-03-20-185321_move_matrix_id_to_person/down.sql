alter table local_user add column matrix_user_id text;

update local_user lu
set matrix_user_id = p.matrix_user_id 
from person p
where p.id = lu.person_id;

alter table person drop column matrix_user_id;
