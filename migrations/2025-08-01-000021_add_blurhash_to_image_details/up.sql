-- Add a blurhash column for image_details
ALTER TABLE image_details
-- Supposed to be 20-30 chars, use 50 to be safe
    ADD COLUMN blurhash varchar(50);

