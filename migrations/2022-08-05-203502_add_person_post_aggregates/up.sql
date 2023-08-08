-- This table stores the # of read comments for a person, on a post
-- It can then be joined to post_aggregates to get an unread count:
-- unread = post_aggregates.comments - person_post_aggregates.read_comments
CREATE TABLE person_post_aggregates (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read_comments bigint NOT NULL DEFAULT 0,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (person_id, post_id)
);

