CREATE TABLE oauth_provider (
    id bigint UNIQUE,
    display_name text NOT NULL,
    issuer text NOT NULL,
    authorization_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    userinfo_endpoint text NOT NULL,
    id_claim text NOT NULL,
    name_claim text NOT NULL,
    client_id text NOT NULL UNIQUE,
    client_secret text NOT NULL,
    scopes text NOT NULL,
    auto_verify_email boolean DEFAULT TRUE NOT NULL,
    auto_approve_application boolean DEFAULT TRUE NOT NULL,
    account_linking_enabled boolean DEFAULT FALSE NOT NULL,
    enabled boolean DEFAULT FALSE NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone,
    PRIMARY KEY (id)
);

ALTER TABLE local_site
    ADD COLUMN oauth_registration boolean DEFAULT FALSE NOT NULL;

