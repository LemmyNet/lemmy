ALTER TABLE post_saved
    DROP CONSTRAINT post_saved_pkey,
    ADD COLUMN id serial PRIMARY KEY;

