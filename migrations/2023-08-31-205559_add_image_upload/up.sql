CREATE TABLE image_upload (
    id serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    pictrs_alias text NOT NULL UNIQUE,
    pictrs_delete_token text NOT NULL,
    published timestamptz DEFAULT now() NOT NULL
);

CREATE INDEX idx_image_upload_local_user_id ON image_upload (local_user_id);

CREATE INDEX idx_image_upload_alias ON image_upload (pictrs_alias);

