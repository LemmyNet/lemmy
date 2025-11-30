-- For each new actions table, create tables that are dropped in up.sql, and insert into them
CREATE TABLE comment_saved (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT comment_saved_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, comment_id)
);

INSERT INTO comment_saved (person_id, comment_id, published)
SELECT
    person_id,
    comment_id,
    saved
FROM
    comment_actions
WHERE
    saved IS NOT NULL;

CREATE TABLE community_block (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    PRIMARY KEY (person_id, community_id)
);

INSERT INTO community_block (person_id, community_id, published)
SELECT
    person_id,
    community_id,
    blocked
FROM
    community_actions
WHERE
    blocked IS NOT NULL;

CREATE TABLE community_person_ban (
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz DEFAULT now(),
    expires timestamptz,
    CONSTRAINT community_user_ban_published_not_null NOT NULL published,
    CONSTRAINT community_user_ban_community_id_not_null NOT NULL community_id,
    CONSTRAINT community_user_ban_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, community_id)
);

INSERT INTO community_person_ban (community_id, person_id, published, expires)
SELECT
    community_id,
    person_id,
    received_ban,
    ban_expires
FROM
    community_actions
WHERE
    received_ban IS NOT NULL;

CREATE TABLE community_moderator (
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT community_moderator_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, community_id)
);

INSERT INTO community_moderator (community_id, person_id, published)
SELECT
    community_id,
    person_id,
    became_moderator
FROM
    community_actions
WHERE
    became_moderator IS NOT NULL;

CREATE TABLE person_block (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    target_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    PRIMARY KEY (person_id, target_id)
);

INSERT INTO person_block (person_id, target_id, published)
SELECT
    person_id,
    target_id,
    blocked
FROM
    person_actions
WHERE
    blocked IS NOT NULL;

CREATE TABLE IF NOT EXISTS person_post_aggregates (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read_comments bigint DEFAULT 0 NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (person_id, post_id)
);

INSERT INTO person_post_aggregates (person_id, post_id, read_comments, published)
SELECT
    person_id,
    post_id,
    read_comments_amount,
    read_comments
FROM
    post_actions
WHERE
    read_comments IS NOT NULL;

CREATE TABLE post_hide (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    PRIMARY KEY (person_id, post_id)
);

INSERT INTO post_hide (post_id, person_id, published)
SELECT
    post_id,
    person_id,
    hidden
FROM
    post_actions
WHERE
    hidden IS NOT NULL;

CREATE TABLE IF NOT EXISTS post_like (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    score smallint NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT post_like_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, post_id)
);

INSERT INTO post_like (post_id, person_id, score, published)
SELECT
    post_id,
    person_id,
    CASE WHEN vote_is_upvote THEN
        1
    ELSE
        -1
    END,
    liked
FROM
    post_actions
WHERE
    liked IS NOT NULL;

CREATE TABLE post_saved (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT post_saved_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, post_id)
);

INSERT INTO post_saved (post_id, person_id, published)
SELECT
    post_id,
    person_id,
    saved
FROM
    post_actions
WHERE
    saved IS NOT NULL;

-- Do the opposite of the `ALTER TABLE` commands in up.sql
DELETE FROM comment_actions
WHERE liked IS NULL;

DELETE FROM community_actions
WHERE followed IS NULL;

DELETE FROM instance_actions
WHERE blocked IS NULL;

DELETE FROM person_actions
WHERE followed IS NULL;

DELETE FROM post_actions
WHERE read IS NULL;

CREATE TABLE IF NOT EXISTS comment_like (
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    score smallint NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT comment_like_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, comment_id)
);

INSERT INTO comment_like (comment_id, person_id, score, published)
SELECT
    comment_id,
    person_id,
    CASE WHEN vote_is_upvote THEN
        1
    ELSE
        -1
    END,
    liked
FROM
    comment_actions
WHERE
    liked IS NOT NULL;

ALTER TABLE community_actions RENAME TO community_follower;

ALTER TABLE instance_actions RENAME TO instance_block;

ALTER TABLE person_actions RENAME TO person_follower;

CREATE TABLE IF NOT EXISTS post_read (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz DEFAULT now() NOT NULL,
    CONSTRAINT post_read_user_id_not_null NOT NULL person_id,
    PRIMARY KEY (person_id, post_id)
);

INSERT INTO post_read (post_id, person_id, published)
SELECT
    post_id,
    person_id,
    read
FROM
    post_actions
WHERE
    read IS NOT NULL;

ALTER TABLE community_follower RENAME COLUMN followed TO published;

ALTER TABLE community_follower RENAME COLUMN follow_state TO state;

ALTER TABLE community_follower RENAME COLUMN follow_approver_id TO approver_id;

ALTER TABLE instance_block RENAME COLUMN blocked TO published;

ALTER TABLE person_follower RENAME COLUMN person_id TO follower_id;

ALTER TABLE person_follower RENAME COLUMN target_id TO person_id;

ALTER TABLE person_follower RENAME COLUMN followed TO published;

ALTER TABLE person_follower RENAME COLUMN follow_pending TO pending;

ALTER TABLE community_follower
    DROP CONSTRAINT community_actions_pkey,
    DROP CONSTRAINT community_actions_check_followed,
    DROP CONSTRAINT community_actions_check_received_ban,
    DROP CONSTRAINT community_actions_community_id_not_null,
    ADD CONSTRAINT community_actions_pkey PRIMARY KEY (person_id, community_id),
    ALTER COLUMN community_id SET NOT NULL,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now(),
    ADD CONSTRAINT community_follower_pending_not_null NOT NULL state,
    DROP COLUMN blocked,
    DROP COLUMN became_moderator,
    DROP COLUMN received_ban,
    DROP COLUMN ban_expires;

ALTER TABLE community_follower RENAME CONSTRAINT community_actions_person_id_not_null TO community_follower_user_id_not_null;

ALTER TABLE instance_block
    DROP CONSTRAINT instance_actions_pkey,
    DROP CONSTRAINT instance_actions_instance_id_not_null,
    DROP CONSTRAINT instance_actions_person_id_not_null,
    ADD CONSTRAINT instance_actions_pkey PRIMARY KEY (person_id, instance_id),
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN instance_id SET NOT NULL,
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now();

ALTER TABLE person_follower
    DROP CONSTRAINT person_actions_pkey,
    DROP CONSTRAINT person_actions_check_followed,
    DROP CONSTRAINT person_actions_person_id_not_null,
    DROP CONSTRAINT person_actions_target_id_not_null,
    ADD CONSTRAINT person_actions_pkey PRIMARY KEY (follower_id, person_id),
    ALTER COLUMN follower_id SET NOT NULL,
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now(),
    ALTER COLUMN pending SET NOT NULL,
    DROP COLUMN blocked;

-- Rename associated stuff
ALTER INDEX community_actions_pkey RENAME TO community_follower_pkey;

ALTER INDEX idx_community_actions_community RENAME TO idx_community_follower_community;

ALTER TABLE community_follower RENAME CONSTRAINT community_actions_community_id_fkey TO community_follower_community_id_fkey;

ALTER TABLE community_follower RENAME CONSTRAINT community_actions_person_id_fkey TO community_follower_person_id_fkey;

ALTER TABLE community_follower RENAME CONSTRAINT community_actions_follow_approver_id_fkey TO community_follower_approver_id_fkey;

ALTER INDEX instance_actions_pkey RENAME TO instance_block_pkey;

ALTER TABLE instance_block RENAME CONSTRAINT instance_actions_instance_id_fkey TO instance_block_instance_id_fkey;

ALTER TABLE instance_block RENAME CONSTRAINT instance_actions_person_id_fkey TO instance_block_person_id_fkey;

ALTER INDEX person_actions_pkey RENAME TO person_follower_pkey;

ALTER TABLE person_follower RENAME CONSTRAINT person_actions_target_id_fkey TO person_follower_person_id_fkey;

ALTER TABLE person_follower RENAME CONSTRAINT person_actions_person_id_fkey TO person_follower_follower_id_fkey;

-- Rename idx_community_actions_followed and remove filter
CREATE INDEX idx_community_follower_published ON community_follower (published);

DROP INDEX idx_community_actions_followed;

-- Move indexes back to their original tables
CREATE INDEX idx_comment_saved_comment ON comment_saved (comment_id);

CREATE INDEX idx_comment_saved_person ON comment_saved (person_id);

CREATE INDEX idx_community_block_community ON community_block (community_id);

CREATE INDEX idx_community_moderator_community ON community_moderator (community_id);

CREATE INDEX idx_community_moderator_published ON community_moderator (published);

CREATE INDEX idx_person_block_person ON person_block (person_id);

CREATE INDEX idx_person_block_target ON person_block (target_id);

CREATE INDEX IF NOT EXISTS idx_person_post_aggregates_person ON person_post_aggregates (person_id);

CREATE INDEX IF NOT EXISTS idx_person_post_aggregates_post ON person_post_aggregates (post_id);

CREATE INDEX IF NOT EXISTS idx_post_like_post ON post_like (post_id);

CREATE INDEX idx_comment_like_comment ON comment_like (comment_id);

CREATE INDEX idx_post_hide_post ON post_hide (post_id);

CREATE INDEX idx_post_read_post ON post_read (post_id);

CREATE INDEX idx_post_saved_post ON post_saved (post_id);

CREATE INDEX idx_post_like_published ON post_like (published);

CREATE INDEX idx_comment_like_published ON comment_like (published);

DROP INDEX idx_person_actions_person, idx_person_actions_target, idx_post_actions_person, idx_post_actions_post;

-- Drop `NOT NULL` indexes of columns that still exist
DROP INDEX idx_comment_actions_liked_not_null, idx_community_actions_followed_not_null, idx_person_actions_followed_not_null, idx_post_actions_read_not_null, idx_instance_actions_blocked_not_null, idx_comment_actions_person, idx_community_actions_person, idx_instance_actions_instance, idx_instance_actions_person;

-- Drop statistics of columns that still exist
DROP statistics comment_actions_liked_stat, community_actions_followed_stat, person_actions_followed_stat;

DROP TABLE comment_actions, post_actions;

