CREATE TABLE remote_image (
    id serial PRIMARY KEY,
    link text not null unique,
    published timestamptz DEFAULT now() NOT NULL
);

alter table image_upload rename to local_image;
