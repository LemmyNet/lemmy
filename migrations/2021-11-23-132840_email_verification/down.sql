-- revert defaults from db for local user init
ALTER TABLE local_user
    ALTER COLUMN theme SET DEFAULT 'darkly';

ALTER TABLE local_user
    ALTER COLUMN default_listing_type SET DEFAULT 1;

-- remove tables and columns for optional email verification
ALTER TABLE site
    DROP COLUMN require_email_verification;

ALTER TABLE local_user
    DROP COLUMN email_verified;

DROP TABLE email_verification;

