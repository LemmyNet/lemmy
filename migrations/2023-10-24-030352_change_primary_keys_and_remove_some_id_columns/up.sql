ALTER TABLE captcha_answer
    DROP COLUMN id,
    ADD PRIMARY KEY (uuid),
    DROP CONSTRAINT captcha_answer_uuid_key;

ALTER TABLE post_saved
    DROP COLUMN id,
    ADD PRIMARY KEY (person_id, post_id),
    DROP CONSTRAINT post_saved_post_id_person_id_key;

DROP INDEX idx_post_saved_person_id;

