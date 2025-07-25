CREATE TABLE captcha_answer (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    uuid uuid NOT NULL UNIQUE DEFAULT gen_random_uuid (),
    answer text NOT NULL,
    published timestamp NOT NULL DEFAULT now()
);

