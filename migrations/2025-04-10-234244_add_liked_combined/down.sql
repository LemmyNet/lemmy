DROP TABLE person_liked_combined;

ALTER TABLE post_actions
    DROP COLUMN person_local;

ALTER TABLE comment_actions
    DROP COLUMN person_local;

