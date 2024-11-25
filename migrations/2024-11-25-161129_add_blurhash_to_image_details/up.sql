-- Add a blurhash column for image_details
ALTER TABLE image_details
-- Supposed to be 20-30 chars, use 50 to be safe
-- TODO this should be made not null for future versions of pictrs
    ADD COLUMN blurhash varchar(50);

