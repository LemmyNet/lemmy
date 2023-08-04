ALTER TABLE local_user
    ADD COLUMN validator_time timestamp NOT NULL DEFAULT now();

