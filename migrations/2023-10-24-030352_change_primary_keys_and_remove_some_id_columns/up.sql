ALTER TABLE post_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (post_id, person_id);

