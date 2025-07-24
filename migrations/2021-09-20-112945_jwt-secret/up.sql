-- generate a jwt secret
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE secret (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    jwt_secret varchar NOT NULL DEFAULT gen_random_uuid ()
);

INSERT INTO secret DEFAULT VALUES;
