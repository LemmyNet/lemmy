ALTER TABLE local_image add column local_user_id int REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

update local_image as li
set local_user_id = lu.id
from local_user as lu
where li.person_id = lu.person_id;

-- Remove the person_id column
ALTER table local_image drop column person_id;

create index idx_image_upload_local_user_id on local_image (local_user_id);
