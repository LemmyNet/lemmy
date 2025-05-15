ALTER TABLE activity
    ADD COLUMN user_id integer;

ALTER TABLE activity
    DROP COLUMN sensitive;

