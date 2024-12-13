-- Creates combined tables for
-- Search: (post, comment, community, person)

CREATE TABLE search_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    -- TODO Need to figure out all the possible sort types, unified into SearchSortType
    -- This is difficult because other than published, there is no unified way to sort them.
    -- 
    -- All have published.
    -- post and comment have top and time-limited scores and ranks.
    -- persons have post and comment counts, and scores (not time-limited).
    -- communities have subscribers, post and comment counts, and active users per X time.
    -- 
    -- I'm thinking just published and score (and use active_monthly users as the community score), is the best way to start.
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int UNIQUE REFERENCES comment ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int UNIQUE REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    person_id int UNIQUE REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1)
);

CREATE INDEX idx_search_combined_published ON search_combined (published DESC, id DESC);

CREATE INDEX idx_search_combined_published_asc ON search_combined (reverse_timestamp_sort (published) DESC, id DESC);

-- Updating the history
INSERT INTO search_combined (published, post_id, comment_id, community_id, person_id)
SELECT
    published,
    id,
    NULL::int,
    NULL::int,
    NULL::int
FROM
    post
UNION ALL
SELECT
    published,
    NULL::int,
    id,
    NULL::int,
    NULL::int
FROM
    comment
UNION ALL
SELECT
    published,
    NULL::int,
    NULL::int,
    id,
    NULL::int
FROM
    community
UNION ALL
SELECT
    published,
    NULL::int,
    NULL::int,
    NULL::int,
    id
FROM
    person;
