DROP TABLE oauth_account;

DROP TABLE oauth_provider;

ALTER TABLE local_site
    DROP COLUMN oauth_registration;

ALTER TABLE local_user
    ALTER COLUMN password_encrypted SET NOT NULL;

