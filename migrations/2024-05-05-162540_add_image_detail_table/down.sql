ALTER TABLE remote_image
    ADD UNIQUE (link),
    DROP CONSTRAINT remote_image_pkey,
    ADD COLUMN id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY;

DROP TABLE image_details;

