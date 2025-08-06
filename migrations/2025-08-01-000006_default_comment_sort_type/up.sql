-- Rename the post sort enum
ALTER TYPE sort_type_enum RENAME TO post_sort_type_enum;

-- Rename the default post sort columns
ALTER TABLE local_user RENAME COLUMN default_sort_type TO default_post_sort_type;

ALTER TABLE local_site RENAME COLUMN default_sort_type TO default_post_sort_type;

-- Create the comment sort type enum
CREATE TYPE comment_sort_type_enum AS ENUM (
    'Hot',
    'Top',
    'New',
    'Old',
    'Controversial'
);

-- Add the new default comment sort columns to local_user and local_site
ALTER TABLE local_user
    ADD COLUMN default_comment_sort_type comment_sort_type_enum NOT NULL DEFAULT 'Hot';

ALTER TABLE local_site
    ADD COLUMN default_comment_sort_type comment_sort_type_enum NOT NULL DEFAULT 'Hot';

