CREATE TABLE remote_image (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    link text NOT NULL UNIQUE,
    published timestamptz DEFAULT now() NOT NULL
);

ALTER TABLE image_upload RENAME TO local_image;

