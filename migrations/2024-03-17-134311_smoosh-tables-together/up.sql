-- For each new actions table:
--   * Transform the table previously used for the most common action type into the new actions table,
--   which should only change the table's metadata instead of rewriting the rows
--   * Add actions from other old tables to the new table
--
-- Create comment_actions from comment_like
ALTER TABLE comment_like RENAME TO comment_actions;

ALTER TABLE comment_actions RENAME COLUMN published TO liked;

ALTER TABLE comment_actions RENAME COLUMN score TO like_score;

ALTER TABLE comment_actions
    ALTER COLUMN post_id DROP NOT NULL,
    ALTER COLUMN liked DROP NOT NULL,
    ALTER COLUMN liked DROP DEFAULT,
    ALTER COLUMN like_score DROP NOT NULL,
    ADD COLUMN saved timestamptz,
    -- `post_id` was only in the `comment_liked` table, and removing it entirely or making it not null
    -- for the `saved` action would make this PR too complicated
    ADD CONSTRAINT comment_actions_check_liked CHECK ((liked IS NULL) = (like_score IS NULL) = (post_id IS NULL));

WITH old_comment_saved AS (
    DELETE FROM comment_saved
RETURNING
    *)
    INSERT INTO comment_actions (person_id, comment_id, saved)
    SELECT
        person_id,
        comment_id,
        published
    FROM
        old_comment_saved
    ON CONFLICT (person_id,
        comment_id)
        DO UPDATE SET
            saved = excluded.saved;

-- Create community_actions from community_follower
ALTER TABLE community_follower RENAME TO community_actions;

ALTER TABLE community_actions RENAME COLUMN published TO followed;

ALTER TABLE community_actions RENAME pending TO follow_pending;

ALTER TABLE community_actions
    ALTER COLUMN followed DROP NOT NULL,
    ALTER COLUMN followed DROP DEFAULT,
    ALTER COLUMN follow_pending DROP NOT NULL,
    -- This `DROP DEFAULT` is done for community follow, but not person follow. It's not a mistake
    -- in this migration. Believe it or not, `pending` only had a default value in community follow.
        ALTER COLUMN follow_pending DROP DEFAULT,
        ADD COLUMN blocked timestamptz,
        ADD COLUMN became_moderator timestamptz,
        ADD COLUMN received_ban timestamptz,
        ADD COLUMN ban_expires timestamptz,
        ADD CONSTRAINT community_actions_check_followed CHECK ((followed IS NULL) = (follow_pending IS NULL)),
        ADD CONSTRAINT community_actions_check_received_ban CHECK ((received_ban IS NULL, ban_expires IS NULL) != (FALSE, TRUE));

WITH old_community_block AS (
    DELETE FROM community_block
RETURNING
    *)
    INSERT INTO community_actions (person_id, community_id, blocked)
    SELECT
        person_id,
        community_id,
        published
    FROM
        old_community_block
    ON CONFLICT (person_id,
        community_id)
        DO UPDATE SET
            person_id = excluded.person_id,
            community_id = excluded.community_id,
            blocked = excluded.blocked;

WITH old_community_moderator AS (
    DELETE FROM community_moderator
RETURNING
    *)
    INSERT INTO community_actions (person_id, community_id, became_moderator)
    SELECT
        person_id,
        community_id,
        published
    FROM
        old_community_moderator
    ON CONFLICT (person_id,
        community_id)
        DO UPDATE SET
            person_id = excluded.person_id,
            community_id = excluded.community_id,
            became_moderator = excluded.became_moderator;

WITH old_community_person_ban AS (
    DELETE FROM community_person_ban
RETURNING
    *)
    INSERT INTO community_actions (person_id, community_id, received_ban, ban_expires)
    SELECT
        person_id,
        community_id,
        published,
        expires
    FROM
        old_community_person_ban
    ON CONFLICT (person_id,
        community_id)
        DO UPDATE SET
            person_id = excluded.person_id,
            community_id = excluded.community_id,
            received_ban = excluded.received_ban,
            ban_expires = excluded.ban_expires;

-- Create instance_actions from instance_block
ALTER TABLE instance_block RENAME TO instance_actions;

ALTER TABLE instance_actions RENAME COLUMN published TO blocked;

ALTER TABLE instance_actions
    ALTER COLUMN blocked DROP NOT NULL,
    ALTER COLUMN blocked DROP DEFAULT;

-- Create person_actions from person_follower
ALTER TABLE person_follower RENAME TO person_actions;

ALTER TABLE person_actions RENAME COLUMN person_id TO target_id;

ALTER TABLE person_actions RENAME COLUMN follower_id TO person_id;

ALTER TABLE person_actions RENAME COLUMN published TO followed;

ALTER TABLE person_actions RENAME COLUMN pending TO follow_pending;

ALTER TABLE person_actions
    ALTER COLUMN followed DROP NOT NULL,
    ALTER COLUMN followed DROP DEFAULT,
    ALTER COLUMN follow_pending DROP NOT NULL,
    ADD COLUMN blocked timestamptz,
    ADD CONSTRAINT person_actions_check_followed CHECK ((followed IS NULL) = (follow_pending IS NULL));

WITH old_person_block AS (
    DELETE FROM person_block
RETURNING
    *)
    INSERT INTO person_actions (person_id, target_id, blocked)
    SELECT
        person_id,
        target_id,
        published
    FROM
        old_person_block
    ON CONFLICT (person_id,
        target_id)
        DO UPDATE SET
            person_id = excluded.person_id,
            target_id = excluded.target_id,
            blocked = excluded.blocked;

-- Create post_actions from post_read
ALTER TABLE post_read RENAME TO post_actions;

ALTER TABLE post_actions RENAME COLUMN published TO read;

ALTER TABLE post_actions
    ALTER COLUMN read DROP NOT NULL,
    ALTER COLUMN read DROP DEFAULT,
    ADD COLUMN read_comments timestamptz,
    ADD COLUMN read_comments_amount bigint,
    ADD COLUMN saved timestamptz,
    ADD COLUMN liked timestamptz,
    ADD COLUMN like_score smallint,
    ADD COLUMN hidden timestamptz,
    ADD CONSTRAINT post_actions_check_read_comments CHECK ((read_comments IS NULL) = (read_comments_amount IS NULL)),
    ADD CONSTRAINT post_actions_check_liked CHECK ((liked IS NULL) = (like_score IS NULL));

WITH old_person_post_aggregates AS (
    DELETE FROM person_post_aggregates
RETURNING
    *)
    INSERT INTO post_actions (person_id, post_id, read_comments, read_comments_amount)
    SELECT
        person_id,
        post_id,
        published,
        read_comments
    FROM
        old_person_post_aggregates
    ON CONFLICT (person_id,
        post_id)
        DO UPDATE SET
            read_comments = excluded.read_comments,
            read_comments_amount = excluded.read_comments_amount;

WITH old_post_hide AS (
    DELETE FROM post_hide
RETURNING
    *)
    INSERT INTO post_actions (person_id, post_id, hidden)
    SELECT
        person_id,
        post_id,
        published
    FROM
        old_post_hide
    ON CONFLICT (person_id,
        post_id)
        DO UPDATE SET
            hidden = excluded.hidden;

WITH old_post_like AS (
    DELETE FROM post_like
RETURNING
    *)
    INSERT INTO post_actions (person_id, post_id, liked, like_score)
    SELECT
        person_id,
        post_id,
        published,
        score
    FROM
        old_post_like
    ON CONFLICT (person_id,
        post_id)
        DO UPDATE SET
            liked = excluded.liked,
            like_score = excluded.like_score;

WITH old_post_saved AS (
    DELETE FROM post_saved
RETURNING
    *)
    INSERT INTO post_actions (person_id, post_id, saved)
    SELECT
        person_id,
        post_id,
        published
    FROM
        old_post_saved
    ON CONFLICT (person_id,
        post_id)
        DO UPDATE SET
            saved = excluded.saved;

-- Drop old tables
DROP TABLE comment_saved, community_block, community_moderator, community_person_ban, person_block, person_post_aggregates, post_hide, post_like, post_saved;

-- Rename associated stuff
ALTER INDEX comment_like_pkey RENAME TO comment_actions_pkey;

ALTER INDEX idx_comment_like_comment RENAME TO idx_comment_actions_comment;

ALTER TABLE comment_actions RENAME CONSTRAINT comment_like_comment_id_fkey TO comment_actions_comment_id_fkey;

ALTER TABLE comment_actions RENAME CONSTRAINT comment_like_person_id_fkey TO comment_actions_person_id_fkey;

ALTER INDEX community_follower_pkey RENAME TO community_actions_pkey;

ALTER INDEX idx_community_follower_community RENAME TO idx_community_actions_community;

ALTER TABLE community_actions RENAME CONSTRAINT community_follower_community_id_fkey TO community_actions_community_id_fkey;

ALTER TABLE community_actions RENAME CONSTRAINT community_follower_person_id_fkey TO community_actions_person_id_fkey;

ALTER INDEX instance_block_pkey RENAME TO instance_actions_pkey;

ALTER TABLE instance_actions RENAME CONSTRAINT instance_block_instance_id_fkey TO instance_actions_instance_id_fkey;

ALTER TABLE instance_actions RENAME CONSTRAINT instance_block_person_id_fkey TO instance_actions_person_id_fkey;

ALTER INDEX person_follower_pkey RENAME TO person_actions_pkey;

ALTER TABLE person_actions RENAME CONSTRAINT person_follower_person_id_fkey TO person_actions_target_id_fkey;

ALTER TABLE person_actions RENAME CONSTRAINT person_follower_follower_id_fkey TO person_actions_person_id_fkey;

ALTER INDEX post_read_pkey RENAME TO post_actions_pkey;

ALTER TABLE post_actions RENAME CONSTRAINT post_read_person_id_fkey TO post_actions_person_id_fkey;

ALTER TABLE post_actions RENAME CONSTRAINT post_read_post_id_fkey TO post_actions_post_id_fkey;

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

CREATE INDEX idx_post_actions_person ON post_actions (person_id);

CREATE INDEX idx_post_actions_post ON post_actions (post_id);

-- Create new indexes, with `OR` being used to allow `IS NOT NULL` filters in queries to use either column in
-- a group (e.g. `liked IS NOT NULL` and `like_score IS NOT NULL` both work)
CREATE INDEX idx_comment_actions_liked_not_null ON comment_actions (person_id, comment_id)
WHERE
    liked IS NOT NULL OR like_score IS NOT NULL;

CREATE INDEX idx_comment_actions_saved_not_null ON comment_actions (person_id, comment_id)
WHERE
    saved IS NOT NULL;

CREATE INDEX idx_community_actions_followed_not_null ON community_actions (person_id, community_id)
WHERE
    followed IS NOT NULL OR follow_pending IS NOT NULL;

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

CREATE INDEX idx_post_actions_read_not_null ON post_actions (person_id, post_id)
WHERE
    read IS NOT NULL;

CREATE INDEX idx_post_actions_read_comments_not_null ON post_actions (person_id, post_id)
WHERE
    read_comments IS NOT NULL OR read_comments IS NOT NULL;

CREATE INDEX idx_post_actions_saved_not_null ON post_actions (person_id, post_id)
WHERE
    saved IS NOT NULL;

CREATE INDEX idx_post_actions_liked_not_null ON post_actions (person_id, post_id)
WHERE
    liked IS NOT NULL OR like_score IS NOT NULL;

CREATE INDEX idx_post_actions_hidden_not_null ON post_actions (person_id, post_id)
WHERE
    hidden IS NOT NULL;

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

CREATE statistics community_actions_followed_stat ON (followed IS NULL), (follow_pending IS NULL)
FROM community_actions;

CREATE statistics person_actions_followed_stat ON (followed IS NULL), (follow_pending IS NULL)
FROM person_actions;

CREATE statistics post_actions_read_comments_stat ON (read_comments IS NULL), (read_comments_amount IS NULL)
FROM post_actions;

CREATE statistics post_actions_liked_stat ON (liked IS NULL), (like_score IS NULL)
FROM post_actions;

