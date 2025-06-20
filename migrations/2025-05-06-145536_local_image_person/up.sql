-- Since local thumbnails could be generated from posts of external users,
-- use the person_id instead of local_user_id for the LocalImage table.
--
-- Also connect the thumbnail to a post id.
--
-- See https://github.com/LemmyNet/lemmy/issues/5564
ALTER TABLE local_image
    ADD COLUMN person_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN thumbnail_for_post_id int REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Update historical person_id columns
-- Note: The local_user_id rows are null for thumbnails, so there's nothing you can do there.
UPDATE
    local_image AS li
SET
    person_id = lu.person_id
FROM
    local_user AS lu
WHERE
    li.local_user_id = lu.id;

-- Remove the local_user_id column
ALTER TABLE local_image
    DROP COLUMN local_user_id;

CREATE INDEX idx_image_upload_person_id ON local_image (person_id);

