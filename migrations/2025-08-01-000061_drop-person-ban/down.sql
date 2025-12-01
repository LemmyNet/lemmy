CREATE TABLE person_ban (
    person_id integer REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamptz DEFAULT now(),
    CONSTRAINT user_ban_user_id_not_null NOT NULL person_id,
    CONSTRAINT user_ban_published_not_null NOT NULL published_at,
    PRIMARY KEY (person_id)
);

