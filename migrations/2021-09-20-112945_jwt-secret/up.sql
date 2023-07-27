-- generate a jwt secret
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE secret (
    id serial PRIMARY KEY,
    jwt_secret varchar NOT NULL DEFAULT gen_random_uuid ()
);

INSERT INTO secret DEFAULT VALUES;
