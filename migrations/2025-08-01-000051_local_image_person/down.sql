ALTER TABLE local_image
    ADD COLUMN local_user_id int REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE;

UPDATE
    local_image AS li
SET
    local_user_id = lu.id
FROM
    local_user AS lu
WHERE
    li.person_id = lu.person_id;

-- You need to have the exact correct column order, so this needs to be re-created
--
-- Rename the table
ALTER TABLE local_image RENAME TO local_image_old;

-- Rename a few constraints
ALTER TABLE local_image_old RENAME CONSTRAINT image_upload_pkey TO image_upload_pkey_old;

-- Create the old one again
CREATE TABLE local_image (
    local_user_id integer,
    pictrs_alias text,
    published timestamp with time zone DEFAULT now(),
    CONSTRAINT image_upload_pictrs_alias_not_null NOT NULL pictrs_alias,
    CONSTRAINT image_upload_published_not_null NOT NULL published
);

ALTER TABLE ONLY local_image
    ADD CONSTRAINT image_upload_pkey PRIMARY KEY (pictrs_alias);

CREATE INDEX idx_image_upload_local_user_id ON local_image USING btree (local_user_id);

ALTER TABLE ONLY local_image
    ADD CONSTRAINT image_upload_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- Insert the data again
INSERT INTO local_image (local_user_id, pictrs_alias, published)
SELECT
    local_user_id,
    pictrs_alias,
    published
FROM
    local_image_old;

DROP TABLE local_image_old;

