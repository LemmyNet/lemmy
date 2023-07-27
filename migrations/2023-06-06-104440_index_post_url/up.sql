-- Make a hard limit of 512 for the post.url column
-- Truncate existing long rows.
UPDATE
    post
SET
    url =
    LEFT (url,
        512)
WHERE
    length(url) > 512;

-- Enforce the limit
ALTER TABLE post
    ALTER COLUMN url TYPE varchar(512);

-- Add the index
CREATE INDEX idx_post_url ON post (url);

