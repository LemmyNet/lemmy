-- Deletes the `reason` column from the `federation_blocklist` table which is added by up.sql
ALTER TABLE federation_blocklist DROP COLUMN reason;
