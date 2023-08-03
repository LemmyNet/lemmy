-- use defaults from db for local user init
ALTER TABLE local_user
    ALTER COLUMN theme SET DEFAULT 'browser';

ALTER TABLE local_user
    ALTER COLUMN default_listing_type SET DEFAULT 2;

-- add tables and columns for optional email verification
ALTER TABLE site
    ADD COLUMN require_email_verification boolean NOT NULL DEFAULT FALSE;

ALTER TABLE local_user
    ADD COLUMN email_verified boolean NOT NULL DEFAULT FALSE;

CREATE TABLE email_verification (
    id serial PRIMARY KEY,
    local_user_id int REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    email text NOT NULL,
    verification_token text NOT NULL
);

