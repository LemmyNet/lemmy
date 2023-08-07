ALTER TABLE activity
    ADD COLUMN user_id INTEGER;

ALTER TABLE activity
    DROP COLUMN sensitive;

