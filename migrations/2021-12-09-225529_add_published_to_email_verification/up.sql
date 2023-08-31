ALTER TABLE email_verification
    ADD COLUMN published timestamp NOT NULL DEFAULT now();

