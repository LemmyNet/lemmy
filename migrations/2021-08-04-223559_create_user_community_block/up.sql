CREATE TABLE person_block (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    target_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (person_id, target_id)
);

CREATE TABLE community_block (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (person_id, community_id)
);

