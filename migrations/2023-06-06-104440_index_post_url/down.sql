-- Change back the column type
ALTER TABLE post
    ALTER COLUMN url TYPE text;

-- Drop the index
DROP INDEX idx_post_url;

