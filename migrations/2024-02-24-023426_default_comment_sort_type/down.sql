-- This file should undo anything in `up.sql`
-- Rename the post sort enum
ALTER TYPE post_sort_type_enum RENAME TO sort_type_enum;

-- Rename the default post sort columns
ALTER TABLE local_user RENAME COLUMN default_post_sort_type TO default_sort_type;

ALTER TABLE local_site RENAME COLUMN default_post_sort_type TO default_sort_type;

-- Create the comment sort type enum
ALTER TABLE local_user
    DROP COLUMN default_comment_sort_type;

ALTER TABLE local_site
    DROP COLUMN default_comment_sort_type;

-- Drop the comment enum
DROP TYPE comment_sort_type_enum;

