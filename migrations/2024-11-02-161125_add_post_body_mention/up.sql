-- Rename the person_mention table to person_comment_mention
ALTER TABLE person_mention RENAME TO person_comment_mention;

-- Create the new post_mention table
CREATE TABLE person_post_mention (
    id serial PRIMARY KEY,
    recipient_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read boolean DEFAULT FALSE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    UNIQUE (recipient_id, post_id)
);
