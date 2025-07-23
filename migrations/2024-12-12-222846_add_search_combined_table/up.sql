-- Creates combined tables for
-- Search: (post, comment, community, person)
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

-- score is used for the top sort
-- For persons: its post score
-- For comments: score,
-- For posts: score,
-- For community: users active monthly
-- Updating the history
CREATE TABLE search_combined AS
SELECT
    published,
    score::int,
    post_id,
    NULL::int AS comment_id,
    NULL::int AS community_id,
    NULL::int AS person_id
FROM
    post_aggregates
UNION ALL
SELECT
    published,
    score::int,
    NULL::int,
    comment_id,
    NULL::int,
    NULL::int
FROM
    comment_aggregates
UNION ALL
SELECT
    published,
    users_active_month::int,
    NULL::int,
    NULL::int,
    community_id,
    NULL::int
FROM
    community_aggregates
UNION ALL
SELECT
    published,
    post_score::int,
    NULL::int,
    NULL::int,
    NULL::int,
    person_id
FROM
    person_aggregates;

-- Add the constraints
ALTER TABLE search_combined
    ADD COLUMN id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN score SET NOT NULL,
    ALTER COLUMN score SET DEFAULT 0,
    ADD CONSTRAINT search_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT search_combined_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT search_combined_community_id_fkey FOREIGN KEY (community_id) REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT search_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD UNIQUE (post_id),
    ADD UNIQUE (comment_id),
    ADD UNIQUE (community_id),
    ADD UNIQUE (person_id),
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1);

CREATE INDEX idx_search_combined_published ON search_combined (published DESC, id DESC);

CREATE INDEX idx_search_combined_published_asc ON search_combined (reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_search_combined_score ON search_combined (score DESC, id DESC);

