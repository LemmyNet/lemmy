-- Add columns to site table
ALTER TABLE site
    DROP COLUMN require_application;

ALTER TABLE site
    DROP COLUMN application_question;

ALTER TABLE site
    DROP COLUMN private_instance;

-- Add pending to local_user
ALTER TABLE local_user
    DROP COLUMN accepted_application;

DROP TABLE registration_application;

