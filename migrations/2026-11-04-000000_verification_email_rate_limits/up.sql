ALTER TABLE local_site_rate_limit
    ADD COLUMN resend_verification_email_max_requests integer NOT NULL DEFAULT 10 CHECK (resend_verification_email_max_requests >= 0),
    ADD COLUMN resend_verification_email_interval_seconds integer NOT NULL DEFAULT 3600 CHECK (resend_verification_email_interval_seconds > 0),
    ADD COLUMN resend_verification_email_per_account_max_requests integer NOT NULL DEFAULT 3 CHECK (resend_verification_email_per_account_max_requests >= 0),
    ADD COLUMN resend_verification_email_per_account_interval_seconds integer NOT NULL DEFAULT 3600 CHECK (resend_verification_email_per_account_interval_seconds > 0);

