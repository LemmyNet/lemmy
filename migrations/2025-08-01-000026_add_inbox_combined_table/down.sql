-- Rename the person_mention table to person_comment_mention
ALTER TABLE person_comment_mention RENAME TO person_mention;

-- Drop the new tables
DROP TABLE person_post_mention, inbox_combined;

