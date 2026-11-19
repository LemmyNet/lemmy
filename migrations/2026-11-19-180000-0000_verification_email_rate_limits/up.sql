ALTER TABLE local_site_rate_limit
    ADD COLUMN resend_verification_email_max_requests integer NOT NULL DEFAULT 10,
    ADD COLUMN resend_verification_email_interval_seconds integer NOT NULL DEFAULT 3600,
    ADD COLUMN resend_verification_email_per_account_max_requests integer NOT NULL DEFAULT 4,
    ADD COLUMN resend_verification_email_per_account_interval_seconds integer NOT NULL DEFAULT 3600;

