-- Create an extra table to hold local user vote display settings
-- Score and Upvote percentage are turned on by default.
CREATE TABLE local_user_vote_display_mode (
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    score boolean DEFAULT TRUE NOT NULL,
    upvotes boolean DEFAULT FALSE NOT NULL,
    downvotes boolean DEFAULT FALSE NOT NULL,
    upvote_percentage boolean DEFAULT TRUE NOT NULL,
    published timestamp with time zone NOT NULL DEFAULT now(),
    updated timestamp with time zone,
    PRIMARY KEY (local_user_id)
);

-- Insert rows for every local user
INSERT INTO local_user_vote_display_mode (local_user_id)
SELECT
    id
FROM
    local_user;

