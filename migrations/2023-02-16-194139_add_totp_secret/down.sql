ALTER TABLE local_user
    DROP COLUMN totp_2fa_secret;

ALTER TABLE local_user
    DROP COLUMN totp_2fa_url;

