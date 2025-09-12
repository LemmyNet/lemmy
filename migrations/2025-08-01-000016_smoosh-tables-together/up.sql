-- Consolidates  all the old tables like post_read, post_like, into post_actions, to reduce joins and increase performance.
-- This creates the tables:
-- post_actions, comment_actions, community_actions, instance_actions, and person_actions.
--
-- comment_actions
CREATE TABLE comment_actions AS
SELECT
    person_id,
    comment_id,
    cast(max(like_score) AS smallint) AS like_score,
    max(liked) AS liked,
    max(saved) AS saved
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

-- Add the constraints
ALTER TABLE comment_actions
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN comment_id SET NOT NULL,
    ADD PRIMARY KEY (person_id, comment_id),
    ADD CONSTRAINT comment_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT comment_actions_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT comment_actions_check_liked CHECK (((liked IS NULL) = (like_score IS NULL)));

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

-- Here's an SO link on merges, but this turned out to be slower than a
-- disabled triggers + disabled primary key + full union select + insert with group by
-- SO link on merges: https://stackoverflow.com/a/74066614/1655478
CREATE TABLE post_actions AS
SELECT
    person_id,
    post_id,
    max(read) AS read,
    max(read_comments) AS read_comments,
    cast(max(read_comments_amount) AS int) AS read_comments_amount,
    max(saved) AS saved,
    max(liked) AS liked,
    cast(max(like_score) AS smallint) AS like_score,
    max(hidden) AS hidden
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

-- Add the constraints
ALTER TABLE post_actions
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN post_id SET NOT NULL,
    ADD PRIMARY KEY (person_id, post_id),
    ADD CONSTRAINT post_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT post_actions_post_id_fkey FOREIGN KEY (post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT post_actions_check_liked CHECK (((liked IS NULL) = (like_score IS NULL))),
    ADD CONSTRAINT post_actions_check_read_comments CHECK (((read_comments IS NULL) = (read_comments_amount IS NULL)));

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

-- community_actions
CREATE TABLE community_actions AS
SELECT
    person_id,
    community_id,
    max(followed) AS followed,
    max(follow_state) AS follow_state,
    max(follow_approver_id) AS follow_approver_id,
    max(blocked) AS blocked,
    max(became_moderator) AS became_moderator,
    max(received_ban) AS received_ban,
    max(ban_expires) AS ban_expires
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

-- Add the constraints
ALTER TABLE community_actions
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN community_id SET NOT NULL,
    ADD PRIMARY KEY (person_id, community_id),
    ADD CONSTRAINT community_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT community_actions_follow_approver_id_fkey FOREIGN KEY (follow_approver_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT community_actions_community_id_fkey FOREIGN KEY (community_id) REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT community_actions_check_followed CHECK ((((followed IS NULL) = (follow_state IS NULL)) AND (NOT ((followed IS NULL) AND (follow_approver_id IS NOT NULL))))),
    ADD CONSTRAINT community_actions_check_received_ban CHECK ((NOT ((received_ban IS NULL) AND (ban_expires IS NOT NULL))));

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

-- instance_actions
CREATE TABLE instance_actions AS
SELECT
    person_id,
    instance_id,
    published AS blocked
FROM
    instance_block;

DROP TABLE instance_block;

-- Add the constraints
ALTER TABLE instance_actions
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN instance_id SET NOT NULL,
    ADD PRIMARY KEY (person_id, instance_id),
    ADD CONSTRAINT instance_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT instance_actions_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

-- This index is currently redundant because instance_actions only has 1 action type, but inconsistency
-- with other tables would make it harder to do everything correctly when adding another action type
CREATE INDEX idx_instance_actions_person ON instance_actions (person_id);

CREATE INDEX idx_instance_actions_instance ON instance_actions (instance_id);

CREATE INDEX idx_instance_actions_blocked_not_null ON instance_actions (person_id, instance_id)
WHERE
    blocked IS NOT NULL;

-- person_actions
CREATE TABLE person_actions AS
SELECT
    person_id,
    target_id,
    max(followed) AS followed,
    cast(max(follow_pending) AS boolean) AS follow_pending,
    max(blocked) AS blocked
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

-- add primary key, foreign keys, and not nulls
ALTER TABLE person_actions
    ALTER COLUMN person_id SET NOT NULL,
    ALTER COLUMN target_id SET NOT NULL,
    ADD PRIMARY KEY (person_id, target_id),
    ADD CONSTRAINT person_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_actions_target_id_fkey FOREIGN KEY (target_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT person_actions_check_followed CHECK (((followed IS NULL) = (follow_pending IS NULL)));

DROP TABLE person_block, person_follower;

CREATE INDEX idx_person_actions_person ON person_actions (person_id);

CREATE INDEX idx_person_actions_target ON person_actions (target_id);

CREATE INDEX idx_person_actions_followed_not_null ON person_actions (person_id, target_id)
WHERE
    followed IS NOT NULL OR follow_pending IS NOT NULL;

CREATE INDEX idx_person_actions_blocked_not_null ON person_actions (person_id, target_id)
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

