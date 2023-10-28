ALTER TABLE post_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (post_id, person_id),
    DROP CONSTRAINT post_saved_post_id_person_id_key,
    ALTER COLUMN post_id DROP NOT NULL,
    ALTER COLUMN person_id DROP NOT NULL;

