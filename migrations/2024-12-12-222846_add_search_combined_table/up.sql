-- Creates combined tables for
-- Search: (post, comment, community, person)
CREATE TABLE search_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    -- This is used for the top sort
    -- For persons: its post score
    -- For comments: score,
    -- For posts: score,
    -- For community: users active monthly
    score bigint NOT NULL DEFAULT 0,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int UNIQUE REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    person_id int UNIQUE REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1)
);

CREATE INDEX idx_search_combined_published ON search_combined (published DESC, id DESC);

CREATE INDEX idx_search_combined_published_asc ON search_combined (reverse_timestamp_sort (published) DESC, id DESC);

-- Updating the history
INSERT INTO search_combined (published, score, post_id, comment_id, community_id, person_id)
SELECT
    p.published,
    score,
    id,
    NULL::int,
    NULL::int,
    NULL::int
FROM
    post p
    INNER JOIN post_aggregates pa ON p.id = pa.post_id
UNION ALL
SELECT
    c.published,
    score,
    NULL::int,
    id,
    NULL::int,
    NULL::int
FROM
    comment c
    INNER JOIN comment_aggregates ca ON c.id = ca.comment_id
UNION ALL
SELECT
    c.published,
    users_active_month,
    NULL::int,
    NULL::int,
    id,
    NULL::int
FROM
    community c
    INNER JOIN community_aggregates ca ON c.id = ca.community_id
UNION ALL
SELECT
    p.published,
    post_score,
    NULL::int,
    NULL::int,
    NULL::int,
    id
FROM
    person p
    INNER JOIN person_aggregates pa ON p.id = pa.person_id;

