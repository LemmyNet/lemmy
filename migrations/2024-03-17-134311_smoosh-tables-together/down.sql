-- For each new actions table:
--   * Create tables that are dropped in up.sql, and insert into them
--   * Do the opposite of the `ALTER TABLE` commands in up.sql, with `DELETE` being used to
--     only keep rows where the preserved action is not null
--
-- Create comment_like from comment_actions
CREATE TABLE comment_saved (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
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

DELETE FROM comment_actions
WHERE liked IS NULL;

ALTER TABLE comment_actions RENAME TO comment_like;

ALTER TABLE comment_like RENAME COLUMN liked TO published;

ALTER TABLE comment_like RENAME COLUMN like_score TO score;

ALTER TABLE comment_like
    DROP CONSTRAINT comment_actions_check_liked,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now(),
    ALTER COLUMN score SET NOT NULL,
    DROP COLUMN saved;

-- Create community_follower from community_actions
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

CREATE TABLE community_moderator (
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
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

CREATE TABLE community_person_ban (
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    expires timestamptz,
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

DELETE FROM community_actions
WHERE followed IS NULL;

ALTER TABLE community_actions RENAME TO community_follower;

ALTER TABLE community_follower RENAME COLUMN followed TO published;

ALTER TABLE community_follower RENAME follow_pending TO pending;

ALTER TABLE community_follower
    DROP CONSTRAINT community_actions_check_followed,
    DROP CONSTRAINT community_actions_check_received_ban,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now(),
    ALTER COLUMN pending SET NOT NULL,
    -- This `SET DEFAULT` is done for community follow, but not person follow. It's not a mistake
    -- in this migration. Believe it or not, `pending` only had a default value in community follow.
        ALTER COLUMN pending SET DEFAULT FALSE,
        DROP COLUMN blocked,
        DROP COLUMN became_moderator,
        DROP COLUMN received_ban,
        DROP COLUMN ban_expires;

-- Create instance_block from instance_actions
DELETE FROM instance_actions
WHERE blocked IS NULL;

ALTER TABLE instance_actions RENAME TO instance_block;

ALTER TABLE instance_block RENAME COLUMN blocked TO published;

ALTER TABLE instance_block
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now();

-- Create person_follower from person_actions
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

DELETE FROM person_actions
WHERE followed IS NULL;

ALTER TABLE person_actions RENAME TO person_follower;

ALTER TABLE person_follower RENAME COLUMN person_id TO follower_id;

ALTER TABLE person_follower RENAME COLUMN target_id TO person_id;

ALTER TABLE person_follower RENAME COLUMN followed TO published;

ALTER TABLE person_follower RENAME COLUMN follow_pending TO pending;

ALTER TABLE person_follower
    DROP CONSTRAINT person_actions_check_followed,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now(),
    ALTER COLUMN pending SET NOT NULL,
    DROP COLUMN blocked;

-- Create post_read from post_actions
CREATE TABLE person_post_aggregates (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    read_comments bigint DEFAULT 0 NOT NULL,
    published timestamptz NOT NULL,
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

CREATE TABLE post_like (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    score smallint NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
    PRIMARY KEY (person_id, post_id)
);

INSERT INTO post_like (post_id, person_id, score, published)
SELECT
    post_id,
    person_id,
    like_score,
    liked
FROM
    post_actions
WHERE
    liked IS NOT NULL;

CREATE TABLE post_saved (
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz DEFAULT now() NOT NULL,
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

DELETE FROM post_actions
WHERE read IS NULL;

ALTER TABLE post_actions RENAME TO post_read;

ALTER TABLE post_read RENAME COLUMN read TO published;

ALTER TABLE post_read
    DROP CONSTRAINT post_actions_check_read_comments,
    DROP CONSTRAINT post_actions_check_liked,
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT now(),
    DROP COLUMN read_comments,
    DROP COLUMN read_comments_amount,
    DROP COLUMN saved,
    DROP COLUMN liked,
    DROP COLUMN like_score,
    DROP COLUMN hidden;

