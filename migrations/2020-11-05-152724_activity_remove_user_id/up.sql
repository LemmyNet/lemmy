ALTER TABLE activity
    DROP COLUMN user_id;

ALTER TABLE activity
    ADD COLUMN sensitive boolean DEFAULT TRUE;

