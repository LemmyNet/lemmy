CREATE TABLE external_auth (
    id serial PRIMARY KEY,
    local_site_id int REFERENCES local_site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    display_name text NOT NULL,
    auth_type varchar(128) NOT NULL,
    auth_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    user_endpoint text NOT NULL,
    id_attribute text NOT NULL,
    issuer text NOT NULL,
    client_id text NOT NULL UNIQUE,
    client_secret text NOT NULL,
    scopes text NOT NULL,
    published timestamptz without time zone DEFAULT now() NOT NULL,
    updated timestamptz without time zone
);

ALTER TABLE local_site
    ADD COLUMN oauth_registration boolean DEFAULT FALSE NOT NULL;

