alter table person add column matrix_user_id text;

update person p
set matrix_user_id = lu.matrix_user_id 
from local_user lu
where p.id = lu.person_id;

alter table local_user drop column matrix_user_id;
