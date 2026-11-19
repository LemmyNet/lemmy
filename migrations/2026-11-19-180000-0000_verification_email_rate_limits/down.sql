ALTER TABLE local_site_rate_limit
    DROP COLUMN resend_verification_email_max_requests,
    DROP COLUMN resend_verification_email_interval_seconds,
    DROP COLUMN resend_verification_email_per_account_max_requests,
    DROP COLUMN resend_verification_email_per_account_interval_seconds;

