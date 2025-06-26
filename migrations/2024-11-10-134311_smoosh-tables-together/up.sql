-- These (and many future migrations) are taking too long to complete for production instances.
-- Due to this, for some large tables (post_like, comment_like, etc), only fill the last month of data.
-- Keep the old tables around, rename them to `_v019` to keep track.
--
-- Since these and many more future migrations need to do the history filling in the background,
-- Create a table to store background history filling status
CREATE TABLE history_status (
    id int GENERATED ALWAYS AS IDENTITY UNIQUE,
    source text NOT NULL,
    dest text NOT NULL,
    last_scanned_id int,
    last_scanned_timestamp timestamptz,
    PRIMARY KEY (source, dest)
);

-- comment_actions
CREATE TABLE comment_actions (
    id int GENERATED ALWAYS AS IDENTITY UNIQUE,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    like_score smallint,
    liked timestamptz,
    saved timestamptz,
    PRIMARY KEY (person_id, comment_id)
);

-- Insert all comment_saved
INSERT INTO comment_actions (person_id, comment_id, saved)
SELECT
    person_id,
    comment_id,
    published
FROM
    comment_saved
ON CONFLICT (person_id,
    comment_id)
    DO UPDATE SET
        saved = excluded.saved;

DROP TABLE comment_saved;

-- Insert only last month from comment_like
ALTER TABLE comment_like RENAME TO comment_like_v019;

INSERT INTO comment_actions (person_id, comment_id, like_score, liked)
SELECT
    person_id,
    comment_id,
    score,
    published
FROM
    comment_like_v019
WHERE
    published > now() - interval '1 month'
ON CONFLICT (person_id,
    comment_id)
    DO UPDATE SET
        liked = excluded.liked,
        like_score = excluded.like_score;

-- Delete that data
DELETE FROM comment_like_v019
WHERE published > now() - interval '1 month';

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('comment_like_v019', 'comment_actions', now() - interval '1 month');

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
    id int GENERATED ALWAYS AS IDENTITY UNIQUE,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    read timestamptz,
    read_comments timestamptz,
    read_comments_amount int,
    saved timestamptz,
    liked timestamptz,
    like_score smallint,
    hidden timestamptz,
    PRIMARY KEY (person_id, post_id)
);

-- post_like, post_read, and person_post_aggregates need history tables
ALTER TABLE post_read RENAME TO post_read_v019;

INSERT INTO post_actions (person_id, post_id, read)
SELECT
    person_id,
    post_id,
    published
FROM
    post_read_v019
WHERE
    published > now() - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        read = excluded.read;

-- Delete that data
DELETE FROM post_read_v019
WHERE published > now() - interval '1 month';

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('post_read_v019', 'post_actions', now() - interval '1 month');

ALTER TABLE person_post_aggregates RENAME TO person_post_aggregates_v019;

INSERT INTO post_actions (person_id, post_id, read_comments, read_comments_amount)
SELECT
    person_id,
    post_id,
    published,
    read_comments
FROM
    person_post_aggregates_v019
WHERE
    published > now() - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        read_comments = excluded.read_comments,
        read_comments_amount = excluded.read_comments_amount;

-- Delete that data
DELETE FROM person_post_aggregates_v019
WHERE published > now() - interval '1 month';

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('person_post_aggregates_v019', 'post_actions', now() - interval '1 month');

ALTER TABLE post_like RENAME TO post_like_v019;

INSERT INTO post_actions (person_id, post_id, liked, like_score)
SELECT
    person_id,
    post_id,
    published,
    score
FROM
    post_like_v019
WHERE
    published > now() - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        liked = excluded.liked,
        like_score = excluded.like_score;

-- Delete that data
DELETE FROM post_like_v019
WHERE published > now() - interval '1 month';

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('post_like_v019', 'post_actions', now() - interval '1 month');

-- insert saved, hidden, read with full history
INSERT INTO post_actions (person_id, post_id, saved)
SELECT
    person_id,
    post_id,
    published
FROM
    post_saved
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        saved = excluded.saved;

DROP TABLE post_saved;

INSERT INTO post_actions (person_id, post_id, hidden)
SELECT
    person_id,
    post_id,
    published
FROM
    post_hide
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        hidden = excluded.hidden;

DROP TABLE post_hide;

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

ALTER TABLE community_follower RENAME TO community_actions;

ALTER TABLE instance_block RENAME TO instance_actions;

ALTER TABLE person_follower RENAME TO person_actions;

ALTER TABLE community_actions RENAME COLUMN published TO followed;

ALTER TABLE community_actions RENAME COLUMN state TO follow_state;

ALTER TABLE community_actions RENAME COLUMN approver_id TO follow_approver_id;

ALTER TABLE instance_actions RENAME COLUMN published TO blocked;

ALTER TABLE person_actions RENAME COLUMN person_id TO target_id;

ALTER TABLE person_actions RENAME COLUMN follower_id TO person_id;

ALTER TABLE person_actions RENAME COLUMN published TO followed;

ALTER TABLE person_actions RENAME COLUMN pending TO follow_pending;

-- Mark all constraints of affected tables as deferrable to speed up migration
ALTER TABLE community_actions
    ALTER CONSTRAINT community_follower_community_id_fkey DEFERRABLE;

ALTER TABLE community_actions
    ALTER CONSTRAINT community_follower_approver_id_fkey DEFERRABLE;

ALTER TABLE community_actions
    ALTER CONSTRAINT community_follower_person_id_fkey DEFERRABLE;

ALTER TABLE instance_actions
    ALTER CONSTRAINT instance_block_instance_id_fkey DEFERRABLE;

ALTER TABLE instance_actions
    ALTER CONSTRAINT instance_block_person_id_fkey DEFERRABLE;

ALTER TABLE person_actions
    ALTER CONSTRAINT person_follower_follower_id_fkey DEFERRABLE;

ALTER TABLE person_actions
    ALTER CONSTRAINT person_follower_person_id_fkey DEFERRABLE;

ALTER TABLE community_actions
    ALTER COLUMN followed DROP NOT NULL,
    ALTER COLUMN followed DROP DEFAULT,
    ALTER COLUMN follow_state DROP NOT NULL,
    ADD COLUMN blocked timestamptz,
    ADD COLUMN became_moderator timestamptz,
    ADD COLUMN received_ban timestamptz,
    ADD COLUMN ban_expires timestamptz;

ALTER TABLE instance_actions
    ALTER COLUMN blocked DROP NOT NULL,
    ALTER COLUMN blocked DROP DEFAULT;

ALTER TABLE person_actions
    ALTER COLUMN followed DROP NOT NULL,
    ALTER COLUMN followed DROP DEFAULT,
    ALTER COLUMN follow_pending DROP NOT NULL,
    ADD COLUMN blocked timestamptz;

INSERT INTO person_actions (person_id, target_id, blocked)
SELECT
    person_id,
    target_id,
    published
FROM
    person_block
ON CONFLICT (person_id,
    target_id)
    DO UPDATE SET
        blocked = excluded.blocked;

UPDATE
    community_actions AS a
SET
    blocked = (
        SELECT
            published
        FROM
            community_block AS b
        WHERE (b.person_id, b.community_id) = (a.person_id, a.community_id)),
became_moderator = (
    SELECT
        published
    FROM
        community_moderator AS b
    WHERE (b.person_id, b.community_id) = (a.person_id, a.community_id)),
(received_ban,
    ban_expires) = (
    SELECT
        published,
        expires
    FROM
        community_person_ban AS b
    WHERE (b.person_id, b.community_id) = (a.person_id, a.community_id));

INSERT INTO community_actions (person_id, community_id, received_ban, ban_expires)
SELECT
    person_id,
    community_id,
    published,
    expires
FROM
    community_person_ban AS b
WHERE
    NOT EXISTS (
        SELECT
        FROM
            community_actions AS a
        WHERE (a.person_id, a.community_id) = (b.person_id, b.community_id));

INSERT INTO community_actions (person_id, community_id, blocked)
SELECT
    person_id,
    community_id,
    published
FROM
    community_block
ON CONFLICT (person_id,
    community_id)
    DO UPDATE SET
        blocked = excluded.blocked
    WHERE
        community_actions.blocked IS NULL;

INSERT INTO community_actions (person_id, community_id, became_moderator)
SELECT
    person_id,
    community_id,
    published
FROM
    community_moderator
ON CONFLICT (person_id,
    community_id)
    DO UPDATE SET
        became_moderator = excluded.became_moderator
    WHERE
        community_actions.became_moderator IS NULL;

-- Drop old tables
DROP TABLE community_block, community_moderator, community_person_ban, person_block;

ALTER INDEX community_follower_pkey RENAME TO community_actions_pkey;

ALTER INDEX idx_community_follower_community RENAME TO idx_community_actions_community;

ALTER TABLE community_actions RENAME CONSTRAINT community_follower_community_id_fkey TO community_actions_community_id_fkey;

ALTER TABLE community_actions RENAME CONSTRAINT community_follower_person_id_fkey TO community_actions_person_id_fkey;

ALTER TABLE community_actions RENAME CONSTRAINT community_follower_approver_id_fkey TO community_actions_follow_approver_id_fkey;

ALTER INDEX instance_block_pkey RENAME TO instance_actions_pkey;

ALTER TABLE instance_actions RENAME CONSTRAINT instance_block_instance_id_fkey TO instance_actions_instance_id_fkey;

ALTER TABLE instance_actions RENAME CONSTRAINT instance_block_person_id_fkey TO instance_actions_person_id_fkey;

ALTER INDEX person_follower_pkey RENAME TO person_actions_pkey;

ALTER TABLE person_actions RENAME CONSTRAINT person_follower_person_id_fkey TO person_actions_target_id_fkey;

ALTER TABLE person_actions RENAME CONSTRAINT person_follower_follower_id_fkey TO person_actions_person_id_fkey;

-- Rename idx_community_follower_published and add filter
CREATE INDEX idx_community_actions_followed ON community_actions (followed)
WHERE
    followed IS NOT NULL;

DROP INDEX idx_community_follower_published;

-- Restore indexes of dropped tables
CREATE INDEX idx_community_actions_became_moderator ON community_actions (became_moderator)
WHERE
    became_moderator IS NOT NULL;

CREATE INDEX idx_person_actions_person ON person_actions (person_id);

CREATE INDEX idx_person_actions_target ON person_actions (target_id);

CREATE INDEX idx_community_actions_followed_not_null ON community_actions (person_id, community_id)
WHERE
    followed IS NOT NULL OR follow_state IS NOT NULL;

CREATE INDEX idx_community_actions_blocked_not_null ON community_actions (person_id, community_id)
WHERE
    blocked IS NOT NULL;

CREATE INDEX idx_community_actions_became_moderator_not_null ON community_actions (person_id, community_id)
WHERE
    became_moderator IS NOT NULL;

CREATE INDEX idx_community_actions_received_ban_not_null ON community_actions (person_id, community_id)
WHERE
    received_ban IS NOT NULL;

CREATE INDEX idx_person_actions_followed_not_null ON person_actions (person_id, target_id)
WHERE
    followed IS NOT NULL OR follow_pending IS NOT NULL;

CREATE INDEX idx_person_actions_blocked_not_null ON person_actions (person_id, target_id)
WHERE
    blocked IS NOT NULL;

-- This index is currently redundant because instance_actions only has 1 action type, but inconsistency
-- with other tables would make it harder to do everything correctly when adding another action type
CREATE INDEX idx_instance_actions_blocked_not_null ON instance_actions (person_id, instance_id)
WHERE
    blocked IS NOT NULL;

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

ALTER TABLE community_actions
    ADD CONSTRAINT community_actions_check_followed CHECK ((followed IS NULL) = (follow_state IS NULL) AND NOT (followed IS NULL AND follow_approver_id IS NOT NULL)),
    ADD CONSTRAINT community_actions_check_received_ban CHECK (NOT (received_ban IS NULL AND ban_expires IS NOT NULL));

ALTER TABLE person_actions
    ADD CONSTRAINT person_actions_check_followed CHECK ((followed IS NULL) = (follow_pending IS NULL));

-- Remove deferrable to restore original db schema
ALTER TABLE community_actions
    ALTER CONSTRAINT community_actions_community_id_fkey NOT DEFERRABLE;

ALTER TABLE community_actions
    ALTER CONSTRAINT community_actions_follow_approver_id_fkey NOT DEFERRABLE;

ALTER TABLE community_actions
    ALTER CONSTRAINT community_actions_person_id_fkey NOT DEFERRABLE;

ALTER TABLE instance_actions
    ALTER CONSTRAINT instance_actions_instance_id_fkey NOT DEFERRABLE;

ALTER TABLE instance_actions
    ALTER CONSTRAINT instance_actions_person_id_fkey NOT DEFERRABLE;

ALTER TABLE person_actions
    ALTER CONSTRAINT person_actions_person_id_fkey NOT DEFERRABLE;

ALTER TABLE person_actions
    ALTER CONSTRAINT person_actions_target_id_fkey NOT DEFERRABLE;

