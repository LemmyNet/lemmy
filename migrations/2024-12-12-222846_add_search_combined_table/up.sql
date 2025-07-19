-- Creates combined tables for
-- Search: (post, comment, community, person)
SET session_replication_role = replica;

CREATE TABLE search_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    -- This is used for the top sort
    -- For persons: its post score
    -- For comments: score,
    -- For posts: score,
    -- For community: users active monthly
    score int NOT NULL DEFAULT 0,
    post_id int UNIQUE REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    comment_id int UNIQUE REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    community_id int UNIQUE REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    person_id int UNIQUE REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE
);

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
INSERT INTO search_combined (published, score, post_id)
SELECT
    published,
    score,
    post_id
FROM
    post_aggregates
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (post_id)
    DO UPDATE SET
        published = excluded.published,
        score = excluded.score;

-- Don't bother with IDs since these are missing from the aggregates tables
-- Also, the aggregates tables are completely removed in a later PR, so just use the source
INSERT INTO history_status (source, dest)
    VALUES ('post', 'search_combined');

INSERT INTO search_combined (published, score, comment_id)
SELECT
    published,
    score,
    comment_id
FROM
    comment_aggregates
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (comment_id)
    DO UPDATE SET
        published = excluded.published,
        score = excluded.score;

INSERT INTO history_status (source, dest)
    VALUES ('comment', 'search_combined');

INSERT INTO search_combined (published, score, community_id)
SELECT
    published,
    users_active_month,
    community_id
FROM
    community_aggregates
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (community_id)
    DO UPDATE SET
        published = excluded.published,
        score = excluded.score;

INSERT INTO history_status (source, dest)
    VALUES ('community', 'search_combined');

INSERT INTO search_combined (published, score, person_id)
SELECT
    published,
    post_score,
    person_id
FROM
    person_aggregates
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id)
    DO UPDATE SET
        published = excluded.published,
        score = excluded.score;

INSERT INTO history_status (source, dest)
    VALUES ('person', 'search_combined');

CREATE INDEX idx_search_combined_published ON search_combined (published DESC, id DESC);

CREATE INDEX idx_search_combined_published_asc ON search_combined (reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_search_combined_score ON search_combined (score DESC, id DESC);

-- Make sure only one of the columns is not null
ALTER TABLE search_combined
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1),
    ALTER CONSTRAINT search_combined_post_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT search_combined_comment_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT search_combined_community_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT search_combined_person_id_fkey NOT DEFERRABLE;

