-- Add back the enable_downvotes column
ALTER TABLE local_site
    ADD COLUMN enable_downvotes boolean DEFAULT TRUE NOT NULL;

-- regenerate their values (from post_downvotes alone)
WITH subquery AS (
    SELECT
        post_downvotes,
        CASE WHEN post_downvotes = 'Disable'::federation_mode_enum THEN
            FALSE
        ELSE
            TRUE
        END
    FROM
        local_site)
UPDATE
    local_site
SET
    enable_downvotes = subquery.case
FROM
    subquery;

-- Drop the new columns
ALTER TABLE local_site
    DROP COLUMN post_upvotes,
    DROP COLUMN post_downvotes,
    DROP COLUMN comment_upvotes,
    DROP COLUMN comment_downvotes;

DROP TYPE federation_mode_enum;

