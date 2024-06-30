CREATE TABLE oauth_account (
    id serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    oauth_provider_id bigint NOT NULL,
    oauth_user_id text NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone,
    UNIQUE (oauth_provider_id, oauth_user_id),
    UNIQUE (oauth_provider_id, local_user_id)
);

