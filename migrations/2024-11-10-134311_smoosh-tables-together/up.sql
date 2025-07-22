-- Consolidates  all the old tables like post_read, post_like, into post_actions, to reduce joins and increase performance.
-- This creates the tables:
-- post_actions, comment_actions, community_actions, instance_actions, and person_actions.
-- comment_actions
CREATE TABLE comment_actions (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    like_score smallint,
    liked timestamptz,
    saved timestamptz
);

-- Disable the triggers temporarily
ALTER TABLE comment_actions DISABLE TRIGGER ALL;

-- Insert all comment_saved
INSERT INTO comment_actions (person_id, comment_id, like_score, liked, saved)
SELECT
    person_id,
    comment_id,
    max(like_score),
    max(liked),
    max(saved)
FROM (
    SELECT
        person_id,
        comment_id,
        score AS like_score,
        published AS liked,
        NULL::timestamptz AS saved
    FROM
        comment_like
    UNION ALL
    SELECT
        person_id,
        comment_id,
        NULL::int,
        NULL::timestamptz,
        published
    FROM
        comment_saved)
GROUP BY
    person_id,
    comment_id;

-- Drop the tables
DROP TABLE comment_saved, comment_like;

-- Re-enable triggers after upserts
ALTER TABLE comment_actions ENABLE TRIGGER ALL;

-- add the primary key
ALTER TABLE comment_actions
    ADD PRIMARY KEY (person_id, comment_id);

-- Create new indexes, with `OR` being used to allow `IS NOT NULL` filters in queries to use either column in
-- a group (e.g. `liked IS NOT NULL` and `like_score IS NOT NULL` both work)
CREATE INDEX idx_comment_actions_person ON comment_actions (person_id);

CREATE INDEX idx_comment_actions_comment ON comment_actions (comment_id);

CREATE INDEX idx_comment_actions_liked_not_null ON comment_actions (person_id, comment_id)
WHERE
    liked IS NOT NULL OR like_score IS NOT NULL;

CREATE INDEX idx_comment_actions_saved_not_null ON comment_actions (person_id, comment_id)
WHERE
    saved IS NOT NULL;

-- Drop deferrables
ALTER TABLE comment_actions
    ADD CONSTRAINT comment_actions_check_liked CHECK (((liked IS NULL) = (like_score IS NULL))),
    ALTER CONSTRAINT comment_actions_comment_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT comment_actions_person_id_fkey NOT DEFERRABLE;

-- post_actions
CREATE TABLE post_actions (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    read timestamptz,
    read_comments timestamptz,
    read_comments_amount int,
    saved timestamptz,
    liked timestamptz,
    like_score smallint,
    hidden timestamptz
);

-- Disable the triggers temporarily
ALTER TABLE post_actions DISABLE TRIGGER ALL;

-- Here's an SO link on merges, but this turned out to be slower than a
-- disabled triggers + disabled primary key + full union select + insert with group by
-- SO link on merges: https://stackoverflow.com/a/74066614/1655478
INSERT INTO post_actions (person_id, post_id, read, read_comments, read_comments_amount, saved, liked, like_score, hidden)
SELECT
    person_id,
    post_id,
    max(read),
    max(read_comments),
    max(read_comments_amount),
    max(saved),
    max(liked),
    max(like_score),
    max(hidden)
FROM (
    SELECT
        person_id,
        post_id,
        published AS read,
        NULL::timestamptz AS read_comments,
        NULL::int AS read_comments_amount,
        NULL::timestamptz AS saved,
        NULL::timestamptz AS liked,
        NULL::int AS like_score,
        NULL::timestamptz AS hidden
    FROM
        post_read
    UNION ALL
    SELECT
        person_id,
        post_id,
        NULL::timestamptz,
        published,
        read_comments,
        NULL::timestamptz,
        NULL::timestamptz,
        NULL::int,
        NULL::timestamptz
    FROM
        person_post_aggregates
    UNION ALL
    SELECT
        person_id,
        post_id,
        NULL::timestamptz,
        NULL::timestamptz,
        NULL::int,
        published,
        NULL::timestamptz,
        NULL::int,
        NULL::timestamptz
    FROM
        post_saved
    UNION ALL
    SELECT
        person_id,
        post_id,
        NULL::timestamptz,
        NULL::timestamptz,
        NULL::int,
        NULL::timestamptz,
        published,
        score,
        NULL::timestamptz
    FROM
        post_like
    UNION ALL
    SELECT
        person_id,
        post_id,
        NULL::timestamptz,
        NULL::timestamptz,
        NULL::int,
        NULL::timestamptz,
        NULL::timestamptz,
        NULL::int,
        published
    FROM
        post_hide)
GROUP BY
    person_id,
    post_id;

-- Drop the tables
DROP TABLE post_read, person_post_aggregates, post_like, post_saved, post_hide;

-- Add the primary key
ALTER TABLE post_actions
    ADD PRIMARY KEY (person_id, post_id);

-- Re-enable triggers after upserts
ALTER TABLE post_actions ENABLE TRIGGER ALL;

-- Create indexes
CREATE INDEX idx_post_actions_person ON post_actions (person_id);

CREATE INDEX idx_post_actions_post ON post_actions (post_id);

CREATE INDEX idx_post_actions_read_not_null ON post_actions (person_id, post_id)
WHERE
    read IS NOT NULL;

CREATE INDEX idx_post_actions_read_comments_not_null ON post_actions (person_id, post_id)
WHERE
    read_comments IS NOT NULL OR read_comments_amount IS NOT NULL;

CREATE INDEX idx_post_actions_saved_not_null ON post_actions (person_id, post_id)
WHERE
    saved IS NOT NULL;

CREATE INDEX idx_post_actions_liked_not_null ON post_actions (person_id, post_id)
WHERE
    liked IS NOT NULL OR like_score IS NOT NULL;

CREATE INDEX idx_post_actions_hidden_not_null ON post_actions (person_id, post_id)
WHERE
    hidden IS NOT NULL;

ALTER TABLE post_actions
    ADD CONSTRAINT post_actions_check_liked CHECK (((liked IS NULL) = (like_score IS NULL))),
    ADD CONSTRAINT post_actions_check_read_comments CHECK (((read_comments IS NULL) = (read_comments_amount IS NULL))),
    ALTER CONSTRAINT post_actions_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT post_actions_post_id_fkey NOT DEFERRABLE;

-- community_actions
CREATE TABLE community_actions (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    followed timestamptz,
    follow_state community_follower_state,
    follow_approver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    blocked timestamptz,
    became_moderator timestamptz,
    received_ban timestamptz,
    ban_expires timestamptz
);

-- disable triggers
ALTER TABLE community_actions DISABLE TRIGGER ALL;

INSERT INTO community_actions (person_id, community_id, followed, follow_state, follow_approver_id, blocked, became_moderator, received_ban, ban_expires)
SELECT
    person_id,
    community_id,
    max(followed),
    max(follow_state),
    max(follow_approver_id),
    max(blocked),
    max(became_moderator),
    max(received_ban),
    max(ban_expires)
FROM (
    SELECT
        person_id,
        community_id,
        published AS followed,
        state AS follow_state,
        approver_id AS follow_approver_id,
        NULL::timestamptz AS blocked,
        NULL::timestamptz AS became_moderator,
        NULL::timestamptz AS received_ban,
        NULL::timestamptz AS ban_expires
    FROM
        community_follower
    UNION ALL
    SELECT
        person_id,
        community_id,
        NULL::timestamptz,
        NULL::community_follower_state,
        NULL::int,
        published,
        NULL::timestamptz,
        NULL::timestamptz,
        NULL::timestamptz
    FROM
        community_block
    UNION ALL
    SELECT
        person_id,
        community_id,
        NULL::timestamptz,
        NULL::community_follower_state,
        NULL::int,
        NULL::timestamptz,
        published,
        NULL::timestamptz,
        NULL::timestamptz
    FROM
        community_moderator
    UNION ALL
    SELECT
        person_id,
        community_id,
        NULL::timestamptz,
        NULL::community_follower_state,
        NULL::int,
        NULL::timestamptz,
        NULL::timestamptz,
        published,
        expires
    FROM
        community_person_ban)
GROUP BY
    person_id,
    community_id;

-- Drop the old tables
DROP TABLE community_follower, community_block, community_moderator, community_person_ban;

-- Re-enable triggers after upserts
ALTER TABLE community_actions ENABLE TRIGGER ALL;

-- add the primary key
ALTER TABLE community_actions
    ADD PRIMARY KEY (person_id, community_id);

-- Create indexes
CREATE INDEX idx_community_actions_person ON community_actions (person_id);

CREATE INDEX idx_community_actions_community ON community_actions (community_id);

CREATE INDEX idx_community_actions_followed ON community_actions (followed)
WHERE
    followed IS NOT NULL;

CREATE INDEX idx_community_actions_followed_not_null ON community_actions (person_id, community_id)
WHERE
    followed IS NOT NULL OR follow_state IS NOT NULL;

CREATE INDEX idx_community_actions_became_moderator ON community_actions (became_moderator)
WHERE
    became_moderator IS NOT NULL;

CREATE INDEX idx_community_actions_became_moderator_not_null ON community_actions (person_id, community_id)
WHERE
    became_moderator IS NOT NULL;

CREATE INDEX idx_community_actions_blocked_not_null ON community_actions (person_id, community_id)
WHERE
    blocked IS NOT NULL;

CREATE INDEX idx_community_actions_received_ban_not_null ON community_actions (person_id, community_id)
WHERE
    received_ban IS NOT NULL;

ALTER TABLE community_actions
    ADD CONSTRAINT community_actions_check_followed CHECK ((((followed IS NULL) = (follow_state IS NULL)) AND (NOT ((followed IS NULL) AND (follow_approver_id IS NOT NULL))))),
    ADD CONSTRAINT community_actions_check_received_ban CHECK ((NOT ((received_ban IS NULL) AND (ban_expires IS NOT NULL)))),
    ALTER CONSTRAINT community_actions_person_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT community_actions_community_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT community_actions_follow_approver_id_fkey NOT DEFERRABLE;

-- instance_actions
CREATE TABLE instance_actions (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    blocked timestamptz
);

-- disable triggers
ALTER TABLE instance_actions DISABLE TRIGGER ALL;

INSERT INTO instance_actions (person_id, instance_id, blocked)
SELECT
    person_id,
    instance_id,
    published
FROM
    instance_block;

DROP TABLE instance_block;

-- Re-enable triggers after upserts
ALTER TABLE instance_actions ENABLE TRIGGER ALL;

-- add the primary key
ALTER TABLE instance_actions
    ADD PRIMARY KEY (person_id, instance_id);

-- This index is currently redundant because instance_actions only has 1 action type, but inconsistency
-- with other tables would make it harder to do everything correctly when adding another action type
CREATE INDEX idx_instance_actions_person ON instance_actions (person_id);

CREATE INDEX idx_instance_actions_instance ON instance_actions (instance_id);

CREATE INDEX idx_instance_actions_blocked_not_null ON instance_actions (person_id, instance_id)
WHERE
    blocked IS NOT NULL;

ALTER TABLE instance_actions
    ALTER CONSTRAINT instance_actions_instance_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT instance_actions_person_id_fkey NOT DEFERRABLE;

-- person_actions
CREATE TABLE person_actions (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    target_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    followed timestamptz,
    follow_pending boolean,
    blocked timestamptz
);

-- disable triggers
ALTER TABLE person_actions DISABLE TRIGGER ALL;

INSERT INTO person_actions (person_id, target_id, followed, follow_pending, blocked)
SELECT
    person_id,
    target_id,
    max(followed),
    cast(max(follow_pending) AS boolean),
    max(blocked)
FROM (
    SELECT
        follower_id AS person_id,
        person_id AS target_id,
        published AS followed,
        pending::int AS follow_pending,
        NULL::timestamptz AS blocked
    FROM
        person_follower
    UNION ALL
    SELECT
        person_id,
        target_id,
        NULL::timestamptz,
        NULL::int,
        published
    FROM
        person_block)
GROUP BY
    person_id,
    target_id;

-- enable triggers
ALTER TABLE person_actions ENABLE TRIGGER ALL;

-- add primary key
ALTER TABLE person_actions
    ADD PRIMARY KEY (person_id, target_id);

DROP TABLE person_block, person_follower;

CREATE INDEX idx_person_actions_person ON person_actions (person_id);

CREATE INDEX idx_person_actions_target ON person_actions (target_id);

CREATE INDEX idx_person_actions_followed_not_null ON person_actions (person_id, target_id)
WHERE
    followed IS NOT NULL OR follow_pending IS NOT NULL;

CREATE INDEX idx_person_actions_blocked_not_null ON person_actions (person_id, target_id)
WHERE
    blocked IS NOT NULL;

ALTER TABLE person_actions
    ALTER CONSTRAINT person_actions_target_id_fkey NOT DEFERRABLE,
    ALTER CONSTRAINT person_actions_person_id_fkey NOT DEFERRABLE,
    ADD CONSTRAINT person_actions_check_followed CHECK (((followed IS NULL) = (follow_pending IS NULL)));

-- Create new statistics for more accurate estimations of how much of an index will be read (e.g. for
-- `(liked, like_score)`, the query planner might othewise assume that `(TRUE, FALSE)` and `(TRUE, TRUE)`
-- are equally likely when only `(TRUE, TRUE)` is possible, which would make it severely underestimate
-- the efficiency of using the index)
CREATE statistics comment_actions_liked_stat ON (liked IS NULL), (like_score IS NULL)
FROM comment_actions;

CREATE statistics community_actions_followed_stat ON (followed IS NULL), (follow_state IS NULL)
FROM community_actions;

CREATE statistics person_actions_followed_stat ON (followed IS NULL), (follow_pending IS NULL)
FROM person_actions;

CREATE statistics post_actions_read_comments_stat ON (read_comments IS NULL), (read_comments_amount IS NULL)
FROM post_actions;

CREATE statistics post_actions_liked_stat ON (liked IS NULL), (like_score IS NULL), (post_id IS NULL)
FROM post_actions;

