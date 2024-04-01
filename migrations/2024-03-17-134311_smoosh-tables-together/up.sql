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
    ALTER COLUMN liked DROP NOT NULL,
    ALTER COLUMN liked DROP DEFAULT,
    ALTER COLUMN like_score DROP NOT NULL,
    ADD COLUMN saved timestamptz,
    ADD CONSTRAINT comment_actions_check_liked CHECK ((liked IS NULL) = (like_score IS NULL));

WITH old_comment_saved AS (
    DELETE FROM comment_saved
RETURNING
    *)
    INSERT INTO comment_actions (person_id, comment_id, saved, post_id)
    SELECT
        old_comment_saved.person_id,
        old_comment_saved.comment_id,
        old_comment_saved.published,
        comment.post_id
    FROM
        old_comment_saved
        INNER JOIN COMMENT ON comment.id = old_comment_saved.comment_id
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

