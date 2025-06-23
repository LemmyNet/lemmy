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
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    community_id int UNIQUE REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    person_id int UNIQUE REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

CREATE INDEX idx_search_combined_published ON search_combined (published DESC, id DESC);

CREATE INDEX idx_search_combined_published_asc ON search_combined (reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_search_combined_score ON search_combined (score DESC, id DESC);

-- Add published to person_aggregates (it was missing for some reason)
ALTER TABLE person_aggregates
    ADD COLUMN published timestamptz NOT NULL DEFAULT now();

UPDATE
    person_aggregates pa
SET
    published = p.published
FROM
    person p
WHERE
    pa.person_id = p.id;

-- Updating the history
INSERT INTO search_combined (published, score, post_id, comment_id, community_id, person_id)
SELECT
    published,
    score,
    post_id,
    NULL::int,
    NULL::int,
    NULL::int
FROM
    post_aggregates
UNION ALL
SELECT
    published,
    score,
    NULL::int,
    comment_id,
    NULL::int,
    NULL::int
FROM
    comment_aggregates
UNION ALL
SELECT
    published,
    users_active_month,
    NULL::int,
    NULL::int,
    community_id,
    NULL::int
FROM
    community_aggregates
UNION ALL
SELECT
    published,
    post_score,
    NULL::int,
    NULL::int,
    NULL::int,
    person_id
FROM
    person_aggregates;

-- Make sure only one of the columns is not null
ALTER TABLE search_combined
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1),
    ALTER CONSTRAINT search_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT search_combined_comment_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT search_combined_community_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT search_combined_person_id_fkey NOT DEFERRABLE;

