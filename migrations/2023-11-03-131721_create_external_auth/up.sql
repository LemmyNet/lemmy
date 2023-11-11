CREATE TABLE external_auth (
    id serial PRIMARY KEY,
    local_site_id int REFERENCES local_site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    display_name text NOT NULL,
    auth_type varchar(128) NOT NULL UNIQUE,
    auth_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    user_endpoint text NOT NULL,
    id_attribute text NOT NULL,
    issuer text NOT NULL,
    client_id text NOT NULL UNIQUE,
    client_secret text NOT NULL,
    scopes text NOT NULL,
    published timestamp without time zone DEFAULT now() NOT NULL,
    updated timestamp without time zone
);

