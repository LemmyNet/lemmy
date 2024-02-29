CREATE TABLE post_hide (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp with time zone NOT NULL DEFAULT now(),
    PRIMARY KEY (person_id, post_id)
);

