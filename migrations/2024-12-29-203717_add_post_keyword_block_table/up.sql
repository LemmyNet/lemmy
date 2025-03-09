CREATE TABLE user_post_keyword_block (
    keyword varchar(50) NOT NULL,
    person_id int REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    PRIMARY KEY (person_id, keyword)
);

