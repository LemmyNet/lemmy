ALTER TABLE captcha_answer
    ADD UNIQUE (uuid),
    DROP CONSTRAINT captcha_answer_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_post_saved_person_id ON post_saved (person_id);

ALTER TABLE post_saved
    ADD UNIQUE (post_id, person_id),
    DROP CONSTRAINT post_saved_pkey,
    ADD COLUMN id serial PRIMARY KEY;

