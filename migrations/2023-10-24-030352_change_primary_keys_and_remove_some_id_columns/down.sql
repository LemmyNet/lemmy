ALTER TABLE captcha_answer
    ADD UNIQUE (uuid),
    DROP CONSTRAINT captcha_answer_pkey,
    ADD COLUMN id serial PRIMARY KEY;

ALTER TABLE comment_aggregates
    ADD UNIQUE (comment_id),
    DROP CONSTRAINT comment_aggregates_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_comment_like_person ON comment_like (person_id);

ALTER TABLE comment_like
    ADD UNIQUE (comment_id, person_id),
    DROP CONSTRAINT comment_like_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_comment_saved_person_id ON comment_saved (person_id);

ALTER TABLE comment_saved
    ADD UNIQUE (comment_id, person_id),
    DROP CONSTRAINT comment_saved_pkey,
    ADD COLUMN id serial PRIMARY KEY;

CREATE INDEX idx_post_saved_person_id ON post_saved (person_id);

ALTER TABLE post_saved
    ADD UNIQUE (post_id, person_id),
    DROP CONSTRAINT post_saved_pkey,
    ADD COLUMN id serial PRIMARY KEY;

