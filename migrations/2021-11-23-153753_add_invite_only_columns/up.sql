-- Add columns to site table
ALTER TABLE site
    ADD COLUMN require_application boolean NOT NULL DEFAULT FALSE;

ALTER TABLE site
    ADD COLUMN application_question text;

ALTER TABLE site
    ADD COLUMN private_instance boolean NOT NULL DEFAULT FALSE;

-- Add pending to local_user
ALTER TABLE local_user
    ADD COLUMN accepted_application boolean NOT NULL DEFAULT FALSE;

CREATE TABLE registration_application (
    id serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    answer text NOT NULL,
    admin_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    deny_reason text,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (local_user_id)
);

CREATE INDEX idx_registration_application_published ON registration_application (published DESC);

