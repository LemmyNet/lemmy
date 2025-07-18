-- Consolidates  all the old tables like post_read, post_like, into post_actions, to reduce joins and increase performance.
-- This creates the tables:
-- post_actions, comment_actions, community_actions, instance_actions, and person_actions.
--
-- Fetching the full history takes too long to complete for production instances.
-- Due to this, for some large tables (post_like, post_read, comment_like, etc), only fill the last month of data.
-- A code migration will handle the rest of the history in the background.
--
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
-- Create an index on published to speed up history updates
CREATE INDEX idx_comment_like_published_desc ON comment_like (published DESC);

INSERT INTO comment_actions (person_id, comment_id, like_score, liked)
SELECT
    person_id,
    comment_id,
    score,
    published
FROM
    comment_like
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id,
    comment_id)
    DO UPDATE SET
        liked = excluded.liked,
        like_score = excluded.like_score;

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('comment_like', 'comment_actions', CURRENT_DATE - interval '1 month');

-- Delete that data
DELETE FROM comment_like
WHERE published > CURRENT_DATE - interval '1 month';

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
-- Create an index on published to speed up history updates
CREATE INDEX idx_post_read_published_desc ON post_read (published DESC);

INSERT INTO post_actions (person_id, post_id, read)
SELECT
    person_id,
    post_id,
    published
FROM
    post_read
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        read = excluded.read;

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('post_read', 'post_actions', CURRENT_DATE - interval '1 month');

-- Delete that data
DELETE FROM post_read
WHERE published > CURRENT_DATE - interval '1 month';

CREATE INDEX idx_person_post_aggregates_published_desc ON person_post_aggregates (published DESC);

INSERT INTO post_actions (person_id, post_id, read_comments, read_comments_amount)
SELECT
    person_id,
    post_id,
    published,
    read_comments
FROM
    person_post_aggregates
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        read_comments = excluded.read_comments,
        read_comments_amount = excluded.read_comments_amount;

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('person_post_aggregates', 'post_actions', CURRENT_DATE - interval '1 month');

-- Delete that data
DELETE FROM person_post_aggregates
WHERE published > CURRENT_DATE - interval '1 month';

CREATE INDEX idx_post_like_published_desc ON post_like (published DESC);

INSERT INTO post_actions (person_id, post_id, liked, like_score)
SELECT
    person_id,
    post_id,
    published,
    score
FROM
    post_like
WHERE
    published > CURRENT_DATE - interval '1 month'
ON CONFLICT (person_id,
    post_id)
    DO UPDATE SET
        liked = excluded.liked,
        like_score = excluded.like_score;

-- Update history status
INSERT INTO history_status (source, dest, last_scanned_timestamp)
    VALUES ('post_like', 'post_actions', CURRENT_DATE - interval '1 month');

-- Delete that data
DELETE FROM post_like
WHERE published > CURRENT_DATE - interval '1 month';

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

-- community_actions
CREATE TABLE community_actions (
    id int GENERATED ALWAYS AS IDENTITY UNIQUE,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    followed timestamptz,
    follow_state community_follower_state,
    follow_approver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
    blocked timestamptz,
    became_moderator timestamptz,
    received_ban timestamptz,
    ban_expires timestamptz,
    PRIMARY KEY (person_id, community_id)
);

INSERT INTO community_actions (person_id, community_id, followed, follow_state, follow_approver_id)
SELECT
    person_id,
    community_id,
    published,
    state,
    approver_id
FROM
    community_follower
ON CONFLICT (person_id,
    community_id)
    DO UPDATE SET
        followed = excluded.followed,
        follow_state = excluded.follow_state,
        follow_approver_id = excluded.follow_approver_id;

INSERT INTO community_actions (person_id, community_id, received_ban, ban_expires)
SELECT
    person_id,
    community_id,
    published,
    expires
FROM
    community_person_ban
ON CONFLICT (person_id,
    community_id)
    DO UPDATE SET
        received_ban = excluded.received_ban,
        ban_expires = excluded.ban_expires;

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
        blocked = excluded.blocked;

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
        became_moderator = excluded.became_moderator;

DROP TABLE community_follower, community_block, community_moderator, community_person_ban;

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
    id int GENERATED ALWAYS AS IDENTITY UNIQUE,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    blocked timestamptz,
    PRIMARY KEY (person_id, instance_id)
);

INSERT INTO instance_actions (person_id, instance_id, blocked)
SELECT
    person_id,
    instance_id,
    published
FROM
    instance_block
ON CONFLICT (person_id,
    instance_id)
    DO UPDATE SET
        blocked = excluded.blocked;

DROP TABLE instance_block;

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
    id int GENERATED ALWAYS AS IDENTITY UNIQUE,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    target_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE NOT NULL,
    followed timestamptz,
    follow_pending boolean,
    blocked timestamptz,
    PRIMARY KEY (person_id, target_id)
);

INSERT INTO person_actions (person_id, target_id, followed, follow_pending)
SELECT
    follower_id,
    person_id,
    published,
    pending
FROM
    person_follower
ON CONFLICT (person_id,
    target_id)
    DO UPDATE SET
        followed = excluded.followed,
        follow_pending = excluded.follow_pending;

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

