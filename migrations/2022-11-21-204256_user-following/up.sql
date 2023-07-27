-- create user follower table with two references to persons
CREATE TABLE person_follower (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    pending boolean NOT NULL,
    UNIQUE (follower_id, person_id)
);

UPDATE
    community_follower
SET
    pending = FALSE
WHERE
    pending IS NULL;

ALTER TABLE community_follower
    ALTER COLUMN pending SET NOT NULL;

