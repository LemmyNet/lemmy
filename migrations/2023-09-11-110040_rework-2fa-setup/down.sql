ALTER TABLE local_user
    ADD COLUMN totp_2fa_url text;

ALTER TABLE local_user
    DROP COLUMN totp_2fa_enabled;

