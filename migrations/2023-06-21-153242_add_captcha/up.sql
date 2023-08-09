CREATE TABLE captcha_answer (
    id serial PRIMARY KEY,
    uuid uuid NOT NULL UNIQUE DEFAULT gen_random_uuid (),
    answer text NOT NULL,
    published timestamp NOT NULL DEFAULT now()
);

