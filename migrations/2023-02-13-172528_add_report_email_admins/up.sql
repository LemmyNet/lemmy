-- Adding a field to email admins for new reports
ALTER TABLE local_site
    ADD COLUMN reports_email_admins boolean NOT NULL DEFAULT FALSE;

