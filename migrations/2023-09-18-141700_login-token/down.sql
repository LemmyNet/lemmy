DROP TABLE login_token;

ALTER TABLE local_user
    ADD COLUMN validator_time timestamptz NOT NULL DEFAULT now();

