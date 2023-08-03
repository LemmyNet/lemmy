-- Add the mod_transfer_community log table
CREATE TABLE mod_transfer_community (
    id serial PRIMARY KEY,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    other_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE,
    when_ timestamp NOT NULL DEFAULT now()
);

