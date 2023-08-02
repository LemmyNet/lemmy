ALTER TABLE local_user
    ADD COLUMN totp_2fa_secret text;

ALTER TABLE local_user
    ADD COLUMN totp_2fa_url text;

