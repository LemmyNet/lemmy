CREATE TABLE login_token (
    id serial PRIMARY KEY,
    token text NOT NULL UNIQUE,
    user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL
);

-- not needed anymore as we invalidate login tokens on password change
ALTER TABLE local_user
    DROP COLUMN validator_time;

