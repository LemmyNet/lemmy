CREATE TABLE person_ban (
    person_id integer PRIMARY KEY REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamptz NOT NULL DEFAULT now()
);

