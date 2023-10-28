ALTER TABLE post_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, post_id),
    DROP CONSTRAINT post_saved_post_id_person_id_key,
    ALTER COLUMN post_id DROP NOT NULL,
    ALTER COLUMN person_id DROP NOT NULL;

DROP INDEX idx_post_saved_person_id;

