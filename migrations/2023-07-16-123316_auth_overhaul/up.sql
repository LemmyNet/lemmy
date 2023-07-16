-- Your SQL goes here
CREATE TABLE auth_refresh_token
(
    id            serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    token         text                                                          NOT NULL DEFAULT encode(digest(gen_random_bytes(1024), 'sha512'), 'hex'),

    last_used     timestamp                                                     NOT NULL DEFAULT now(),
    last_ip       text                                                          NOT NULL
);

CREATE INDEX idx_auth_refresh_token_token ON auth_refresh_token (token);


CREATE TABLE auth_api_token
(
    id            serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    label         text                                                          NOT NULL,
    token         text                                                          NOT NULL DEFAULT 'lemmyv1_' ||
                                                                                                 encode(digest(gen_random_bytes(1024), 'sha512'), 'hex'),
    expires       timestamp                                                     NOT NULL DEFAULT now(),
    last_used     timestamp                                                     NOT NULL DEFAULT now(),
    last_ip       text                                                          NOT NULL
);


CREATE INDEX idx_auth_api_token_token ON auth_api_token (token);