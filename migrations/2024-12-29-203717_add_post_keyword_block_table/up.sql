CREATE TABLE post_keyword_block (
    id serial PRIMARY KEY,
    keyword varchar(255) NOT NULL,
    person_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL
);

CREATE INDEX idx_post_keyword_block_person_id ON post_keyword_block (person_id);

