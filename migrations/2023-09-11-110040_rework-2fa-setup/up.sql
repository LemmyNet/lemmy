ALTER TABLE local_user
    DROP COLUMN totp_2fa_url;

ALTER TABLE local_user
    ADD COLUMN totp_2fa_enabled boolean NOT NULL DEFAULT FALSE;

