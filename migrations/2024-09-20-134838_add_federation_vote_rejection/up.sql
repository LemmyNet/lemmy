-- This removes the simple enable_downvotes setting, in favor of an
-- expanded federation mode type for post/comment up/downvotes.
-- Create the federation mode enum
CREATE TYPE federation_mode_enum AS ENUM (
    'All',
    'Local',
    'Disable'
);

-- Add the new columns
ALTER TABLE local_site
    ADD COLUMN post_upvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL,
    ADD COLUMN post_downvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL,
    ADD COLUMN comment_upvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL,
    ADD COLUMN comment_downvotes federation_mode_enum DEFAULT 'All'::federation_mode_enum NOT NULL;

-- Copy over the enable_downvotes into the post and comment downvote settings
WITH subquery AS (
    SELECT
        enable_downvotes,
        CASE WHEN enable_downvotes = TRUE THEN
            'All'::federation_mode_enum
        ELSE
            'Disable'::federation_mode_enum
        END
    FROM
        local_site)
UPDATE
    local_site
SET
    post_downvotes = subquery.case,
    comment_downvotes = subquery.case
FROM
    subquery;

-- Drop the enable_downvotes column
ALTER TABLE local_site
    DROP COLUMN enable_downvotes;

