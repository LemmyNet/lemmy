-- Drop the id column from the remote_image table, just use link
ALTER TABLE remote_image
    DROP COLUMN id,
    ADD PRIMARY KEY (link),
    DROP CONSTRAINT remote_image_link_key;

-- No good way to do references here unfortunately, unless we combine the images tables
-- The link should be the URL, not the pictrs_alias, to allow joining from post.thumbnail_url
CREATE TABLE image_details (
    link text PRIMARY KEY,
    width integer NOT NULL,
    height integer NOT NULL,
    content_type text NOT NULL,
    published timestamptz DEFAULT now() NOT NULL
);

