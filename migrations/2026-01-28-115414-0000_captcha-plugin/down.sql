CREATE TABLE captcha_answer (
    uuid uuid NOT NULL DEFAULT gen_random_uuid () PRIMARY KEY,
    answer text NOT NULL,
    published timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE captcha_answer RENAME COLUMN published TO published_at;

ALTER TABLE local_site
    ADD COLUMN captcha_enabled boolean DEFAULT FALSE NOT NULL;

ALTER TABLE local_site
    ADD COLUMN captcha_difficulty varchar(255) DEFAULT 'medium'::character varying NOT NULL;

