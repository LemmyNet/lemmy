ALTER TABLE local_user
    ALTER COLUMN password_encrypted DROP NOT NULL;

CREATE TABLE oauth_provider (
    id serial PRIMARY KEY,
    display_name text NOT NULL,
    issuer text NOT NULL,
    authorization_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    userinfo_endpoint text NOT NULL,
    id_claim text NOT NULL,
    client_id text NOT NULL UNIQUE,
    client_secret text NOT NULL,
    scopes text NOT NULL,
    auto_verify_email boolean DEFAULT TRUE NOT NULL,
    account_linking_enabled boolean DEFAULT FALSE NOT NULL,
    enabled boolean DEFAULT TRUE NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone
);

ALTER TABLE local_site
    ADD COLUMN oauth_registration boolean DEFAULT FALSE NOT NULL;

CREATE TABLE oauth_account (
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    oauth_provider_id int REFERENCES oauth_provider ON UPDATE CASCADE ON DELETE RESTRICT NOT NULL,
    oauth_user_id text NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone,
    UNIQUE (oauth_provider_id, oauth_user_id),
    PRIMARY KEY (oauth_provider_id, local_user_id)
);

