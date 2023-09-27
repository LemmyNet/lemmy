CREATE TABLE login_token (
    id serial PRIMARY KEY,
    token text NOT NULL UNIQUE,
    user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    ip text,
    user_agent text
);

CREATE INDEX idx_login_token_user ON login_token (user_id);

CREATE INDEX idx_login_token_user_token ON login_token (user_id, token);

-- not needed anymore as we invalidate login tokens on password change
ALTER TABLE local_user
    DROP COLUMN validator_time;

