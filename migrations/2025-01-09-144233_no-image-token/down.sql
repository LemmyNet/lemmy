ALTER TABLE local_image
    ADD COLUMN pictrs_delete_token text NOT NULL DEFAULT '';

ALTER TABLE local_image
    ALTER COLUMN pictrs_delete_token DROP DEFAULT;

ALTER TABLE local_image
    ADD COLUMN published_new timestamp with time zone DEFAULT now() NOT NULL;

UPDATE
    local_image
SET
    published_new = published;

ALTER TABLE local_image
    DROP COLUMN published;

ALTER TABLE local_image RENAME published_new TO published;

