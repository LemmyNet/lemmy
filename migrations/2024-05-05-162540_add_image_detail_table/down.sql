ALTER TABLE remote_image
    ADD UNIQUE (link),
    DROP CONSTRAINT remote_image_pkey,
    ADD COLUMN id serial PRIMARY KEY;

DROP TABLE image_details;

