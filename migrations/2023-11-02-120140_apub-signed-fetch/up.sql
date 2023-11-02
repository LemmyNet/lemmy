ALTER TABLE local_site
    ADD COLUMN federation_signed_fetch boolean NOT NULL DEFAULT FALSE;

