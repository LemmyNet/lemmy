-- Since local thumbnails could be generated from posts of external users,
-- use the person_id instead of local_user_id for the LocalImage table.
-- 
-- Seehttps://github.com/LemmyNet/lemmy/issues/5564

ALTER TABLE local_image add column person_id int not null default 0 references person (id)  ON UPDATE CASCADE ON DELETE CASCADE;

-- Update historical person_id columns
-- Note: The local_user_id rows are null for thumbnails, so there's nothing you can do there.

update local_image as li
set person_id = lu.person_id
from local_user as lu
where li.local_user_id = lu.id;

-- Remove the default
ALTER table local_image alter column person_id drop default;

-- Remove the local_user_id column
ALTER table local_image drop column local_user_id;

create index idx_image_upload_person_id on local_image (person_id);
