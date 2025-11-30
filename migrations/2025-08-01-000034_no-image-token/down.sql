ALTER TABLE local_image
    ADD COLUMN pictrs_delete_token text DEFAULT '',
    ADD CONSTRAINT image_upload_pictrs_delete_token_not_null NOT NULL pictrs_delete_token;

ALTER TABLE local_image
    ALTER COLUMN pictrs_delete_token DROP DEFAULT;

ALTER TABLE local_image
    ADD COLUMN published_new timestamp with time zone DEFAULT now();

UPDATE
    local_image
SET
    published_new = published;

ALTER TABLE local_image
    DROP COLUMN published;

ALTER TABLE local_image RENAME published_new TO published;

ALTER TABLE local_image
    ADD CONSTRAINT image_upload_published_not_null NOT NULL published;

