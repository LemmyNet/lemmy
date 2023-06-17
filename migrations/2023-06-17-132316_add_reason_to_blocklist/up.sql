-- This adds a `reason` text column to the `federation_blocklist` table

ALTER TABLE federation_blocklist ADD COLUMN reason TEXT;
