-- Creates combined tables for
-- Profile: (comment, post)
CREATE TABLE profile_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK ((post_id IS NOT NULL)::integer + (comment_id IS NOT NULL)::integer = 1)
);

CREATE INDEX idx_profile_combined_published ON profile_combined (published DESC, id DESC);

CREATE INDEX idx_profile_combined_published_asc ON profile_combined (reverse_timestamp_sort (published) DESC, id DESC);

-- Updating the history
INSERT INTO profile_combined (published, post_id)
SELECT
    published,
    id
FROM
    post;

INSERT INTO profile_combined (published, comment_id)
SELECT
    published,
    id
FROM
    comment;

