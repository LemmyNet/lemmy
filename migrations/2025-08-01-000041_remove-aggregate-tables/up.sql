CREATE FUNCTION get_score (non_1_upvotes integer, non_0_downvotes integer)
    RETURNS integer
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN coalesce (
        non_1_upvotes, 1) - coalesce (
        non_0_downvotes, 0
);

CREATE FUNCTION get_controversy_rank (non_1_upvotes integer, non_0_downvotes integer)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN non_0_downvotes IS NULL
        OR non_1_upvotes IS NOT DISTINCT FROM 0 THEN
        -- upvotes = 0 or downvotes = 0
        0
    WHEN non_1_upvotes IS NULL THEN
        -- upvotes = 1 and downvotes > 0
        (
            1 + non_0_downvotes) ^ (
            1 / non_0_downvotes::real)
    ELSE
        -- non_1_upvotes and non_0_downvotes are not null
        (
            non_1_upvotes + non_0_downvotes) ^ (
            CASE WHEN non_1_upvotes > non_0_downvotes THEN
                non_0_downvotes::real / non_1_upvotes::real
            ELSE
                non_1_upvotes::real / non_0_downvotes::real
            END)
    END;

CREATE FUNCTION inner_get_hot_rank (clamped_score_plus_2 integer, non_null_age smallint)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN
    -- Use greatest(2,score+2), so that the hot_rank will be positive and not ignored.
    -- TODO: figure out why this doesn't work without greatest(2, _)
    log (
        greatest (2, clamped_score_plus_2)) * power (((
        -- Age in hours
        non_null_age::real / 60) + 2), -1.8
);

-- after a week, hot rank will default to 0.
CREATE FUNCTION get_hot_rank (non_1_upvotes integer, non_0_downvotes integer, age smallint)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN age IS NULL THEN
        0
    ELSE
        inner_get_hot_rank (
            2 + greatest (0, get_score (non_1_upvotes, non_0_downvotes)), age)
    END;

CREATE FUNCTION get_scaled_rank (non_1_upvotes integer, non_0_downvotes integer, age smallint, non_0_community_interactions_month integer)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    -- Add 2 to avoid divide by zero errors
    -- Default for score = 1, active users = 1, and now, is (0.1728 / log(2 + 1)) = 0.3621
    -- There may need to be a scale factor multiplied to interactions_month, to make
    -- the log curve less pronounced. This can be tuned in the future.
    RETURN CASE WHEN age IS NULL THEN
        0
    ELSE
        inner_get_hot_rank (
            2 + greatest (0, get_score (non_1_upvotes, non_0_downvotes)), age) / log (
            CASE WHEN non_0_community_interactions_month IS NULL THEN
                2
            ELSE
                2 + non_0_community_interactions_month
            END)
    END;

CREATE FUNCTION get_community_hot_rank (non_1_subscribers integer, age smallint)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN age IS NULL THEN
        0
    ELSE
        inner_get_hot_rank (
            CASE WHEN non_1_subscribers IS NULL THEN
                3
            ELSE
                2 + greatest (0, non_1_subscribers)
            END, age)
    END;

-- if the post is from the future, set age to null. otherwise you can game the post to
-- always be on top even with only 1 vote by setting it to the future
CREATE FUNCTION inner_age (minutes numeric)
    RETURNS smallint
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN minutes >= 0
        AND minutes <= 10080 THEN
        minutes::smallint
    ELSE
        NULL
    END;

CREATE FUNCTION age_of (t timestamp with time zone)
    RETURNS smallint
    LANGUAGE sql
    -- `STABLE PARALLEL SAFE` is correct for `now()` based on the output of `SELECT provolatile, proparallel FROM pg_proc WHERE proname = 'now'`
    STABLE PARALLEL SAFE RETURN inner_age (
extract(minutes FROM (now() - t))
);

-- Merge comment_aggregates into comment table
ALTER TABLE comment
    ADD COLUMN non_1_upvotes int,
    ADD COLUMN non_0_downvotes int,
    ADD COLUMN non_0_child_count int,
    -- Unlike with other columns, a null value for `age` does not represent one specific default value, but instead indicates that the comment is either supposedly from the future or exceeds the maximum age for hot ranks.
    ADD COLUMN age smallint,
    ADD COLUMN non_0_report_count smallint,
    ADD COLUMN non_0_unresolved_report_count smallint;

-- Default value only for future rows, not for already existing rows
ALTER TABLE comment
    ALTER COLUMN non_1_upvotes SET DEFAULT 0;

-- Disable the triggers temporarily
ALTER TABLE comment DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'comment');

UPDATE
    comment
SET
    -- The values that columns typically have shortly after insertion are stored as null so they don't take up space.
    non_1_upvotes = nullif (ca.upvotes, 1),
    non_0_downvotes = nullif (ca.downvotes, 0),
    non_0_child_count = nullif (ca.child_count, 0),
    age = CASE WHEN comment_is_young THEN
        new_age
    ELSE
        NULL
    END,
    non_0_report_count = nullif (ca.report_count, 0),
    non_0_unresolved_report_count = nullif (ca.unresolved_report_count, 0)
FROM
    comment_aggregates AS ca,
    LATERAL (
        SELECT
            extract(minutes FROM (now() - ca.published)) AS new_age),
    LATERAL (
        SELECT
            new_age >= 0
            AND new_age <= 10080 AS comment_is_young)
WHERE
    comment.id = ca.comment_id
    AND (ca.upvotes != 1
        OR ca.downvotes != 0
        OR ca.child_count != 0
        OR ca.report_count != 0
        OR ca.unresolved_report_count != 0
        OR comment_is_young);

DROP TABLE comment_aggregates;

-- Re-enable triggers after upserts
ALTER TABLE comment ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'comment');

-- reindex
REINDEX TABLE comment;

-- 30s-2m each
CREATE INDEX idx_comment_controversy ON comment USING btree (get_controversy_rank (non_1_upvotes, non_0_downvotes) DESC);

CREATE INDEX idx_comment_hot ON comment USING btree (get_hot_rank (non_1_upvotes, non_0_downvotes, age) DESC, get_score (non_1_upvotes, non_0_downvotes) DESC);

CREATE INDEX idx_comment_young ON comment USING btree (published)
WHERE (age IS NOT NULL);

--CREATE INDEX idx_comment_published on comment USING btree (published DESC);
CREATE INDEX idx_comment_score ON comment USING btree (get_score (non_1_upvotes, non_0_downvotes) DESC);

-- merge post_aggregates into post table
ALTER TABLE post
    ADD COLUMN newest_comment_time_necro_after_published timestamp with time zone, -- TODO: remove if unused
    ADD COLUMN newest_comment_time_after_published timestamp with time zone,
    ADD COLUMN non_0_community_interactions_month int, -- TODO: update this when it's updated in community
    ADD COLUMN non_0_comments int,
    ADD COLUMN non_1_upvotes int,
    ADD COLUMN non_0_downvotes int,
    ADD COLUMN age smallint,
    ADD COLUMN newest_non_necro_comment_age smallint,
    ADD COLUMN non_0_report_count smallint,
    ADD COLUMN non_0_unresolved_report_count smallint;

-- Default value only for future rows, not for already existing rows
ALTER TABLE post
    ALTER COLUMN non_1_upvotes SET DEFAULT 0;

-- Disable the triggers temporarily
ALTER TABLE post DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'post');

UPDATE
    post
SET
    newest_comment_time_necro_after_published = nullif (pa.newest_comment_time_necro, pa.published),
    newest_comment_time_after_published = nullif (pa.newest_comment_time, pa.published),
    non_0_community_interactions_month = (
        SELECT
            ca.interactions_month
        FROM
            community_aggregates AS ca
        WHERE
            ca.community_id = pa.community_id
            AND post_is_young
            AND ca.interactions_month != 0),
    non_0_comments = nullif (pa.comments, 0),
    non_1_upvotes = nullif (pa.upvotes, 1),
    non_0_downvotes = nullif (pa.downvotes, 0),
    age = CASE WHEN post_is_young THEN
        new_age
    ELSE
        NULL
    END,
    newest_non_necro_comment_age = CASE WHEN comment_is_young THEN
        new_newest_non_necro_comment_age
    ELSE
        NULL
    END,
    non_0_report_count = nullif (pa.report_count, 0),
    non_0_unresolved_report_count = nullif (pa.unresolved_report_count, 0)
FROM
    post_aggregates AS pa,
    LATERAL (
        SELECT
            extract(minutes FROM (now() - pa.published)) AS new_age,
            extract(minutes FROM (now() - pa.newest_comment_time_necro)) AS new_newest_non_necro_comment_age),
    LATERAL (
        SELECT
            new_age >= 0
            -- maybe the wrong number
            AND new_age <= 10080 AS post_is_young,
            new_newest_non_necro_comment_age >= 0
            AND new_newest_non_necro_comment_age <= 10080 AS comment_is_young)
WHERE
    post.id = pa.post_id
    AND (pa.newest_comment_time_necro != pa.published
        OR pa.newest_comment_time != pa.published
        -- no need to separately check `community_interactions_month` here because of how the subselect uses `post_is_young`
        OR pa.comments != 0
        OR pa.upvotes != 1
        OR pa.downvotes != 0
        OR post_is_young
        OR comment_is_young
        OR pa.report_count != 0
        OR pa.unresolved_report_count != 0);

-- Delete that data
DROP TABLE post_aggregates;

-- Re-enable triggers after upserts
ALTER TABLE post ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'post');

-- reindex
REINDEX TABLE post;

CREATE INDEX idx_post_community_active ON post USING btree (community_id, featured_local DESC, get_hot_rank (non_1_upvotes, non_0_downvotes, coalesce(newest_non_necro_comment_age, age)), published DESC, id DESC);

CREATE INDEX idx_post_community_controversy ON post USING btree (community_id, featured_local DESC, get_controversy_rank (non_1_upvotes, non_0_downvotes) DESC, id DESC);

CREATE INDEX idx_post_community_hot ON post USING btree (community_id, featured_local DESC, get_hot_rank (non_1_upvotes, non_0_downvotes, age) DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_most_comments ON post USING btree (community_id, featured_local DESC, coalesce(non_0_comments, 0) DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time ON post USING btree (community_id, featured_local DESC, coalesce(newest_comment_time_after_published, published) DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post USING btree (community_id, featured_local DESC, coalesce(newest_comment_time_necro_after_published, published) DESC, id DESC);

-- INDEX idx_post_community_published ON post USING btree (community_id, featured_local DESC, published DESC);
--CREATE INDEX idx_post_community_published_asc ON post USING btree (community_id, featured_local DESC, reverse_timestamp_sort (published) DESC);
CREATE INDEX idx_post_community_scaled ON post USING btree (community_id, featured_local DESC, get_scaled_rank (non_1_upvotes, non_0_downvotes, age, non_0_community_interactions_month) DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_score ON post USING btree (community_id, featured_local DESC, get_score (non_1_upvotes, non_0_downvotes) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_active ON post USING btree (community_id, featured_community DESC, get_hot_rank (non_1_upvotes, non_0_downvotes, coalesce(newest_non_necro_comment_age, age)) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_controversy ON post USING btree (community_id, featured_community DESC, get_controversy_rank (non_1_upvotes, non_0_downvotes) DESC, id DESC);

CREATE INDEX idx_post_featured_community_hot ON post USING btree (community_id, featured_community DESC, get_hot_rank (non_1_upvotes, non_0_downvotes, age) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_most_comments ON post USING btree (community_id, featured_community DESC, coalesce(non_0_comments, 0) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time ON post USING btree (community_id, featured_community DESC, coalesce(newest_comment_time_after_published, published) DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time_necr ON post USING btree (community_id, featured_community DESC, coalesce(newest_comment_time_necro_after_published, published) DESC, id DESC);

--CREATE INDEX idx_post_featured_community_published ON post USING btree (community_id, featured_community DESC, published DESC);
CREATE INDEX idx_post_featured_community_published_asc ON post USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_community_scaled ON post USING btree (community_id, featured_community DESC, get_scaled_rank (non_1_upvotes, non_0_downvotes, age, non_0_community_interactions_month) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_score ON post USING btree (community_id, featured_community DESC, get_score (non_1_upvotes, non_0_downvotes) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_active ON post USING btree (featured_local DESC, get_hot_rank (non_1_upvotes, non_0_downvotes, coalesce(newest_non_necro_comment_age, age)) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_controversy ON post USING btree (featured_local DESC, get_controversy_rank (non_1_upvotes, non_0_downvotes) DESC, id DESC);

CREATE INDEX idx_post_featured_local_hot ON post USING btree (featured_local DESC, get_hot_rank (non_1_upvotes, non_0_downvotes, age) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_most_comments ON post USING btree (featured_local DESC, coalesce(non_0_comments, 0) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time ON post USING btree (featured_local DESC, coalesce(newest_comment_time_after_published, published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time_necro ON post USING btree (featured_local DESC, coalesce(newest_comment_time_necro_after_published, published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_published ON post USING btree (featured_local DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_published_asc ON post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_scaled ON post USING btree (featured_local DESC, get_scaled_rank (non_1_upvotes, non_0_downvotes, age, non_0_community_interactions_month) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_score ON post USING btree (featured_local DESC, get_score (non_1_upvotes, non_0_downvotes) DESC, published DESC, id DESC);

CREATE INDEX idx_post_young ON post USING btree (published DESC)
WHERE
    age IS NOT NULL OR newest_non_necro_comment_age IS NOT NULL;

CREATE INDEX idx_post_published_asc ON post USING btree (reverse_timestamp_sort (published) DESC);

-- merge community_aggregates into community table
ALTER TABLE community
    ADD COLUMN non_1_subscribers int,
    ADD COLUMN non_0_posts int,
    ADD COLUMN non_0_comments int,
    ADD COLUMN non_0_users_active_day int,
    ADD COLUMN non_0_users_active_week int,
    ADD COLUMN non_0_users_active_month int,
    ADD COLUMN non_0_users_active_half_year int,
    ADD COLUMN non_0_subscribers_local int,
    ADD COLUMN non_0_interactions_month int,
    ADD COLUMN age smallint,
    ADD COLUMN non_0_report_count smallint,
    ADD COLUMN non_0_unresolved_report_count smallint;

-- Default value only for future rows, not for already existing rows
ALTER TABLE community
    ALTER COLUMN non_1_subscribers SET DEFAULT 0;

-- Disable the triggers temporarily
ALTER TABLE community DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'community');

UPDATE
    community
SET
    non_1_subscribers = nullif (ca.subscribers, 1),
    non_0_posts = nullif (ca.posts, 0),
    non_0_comments = nullif (ca.comments, 0),
    non_0_users_active_day = nullif (ca.users_active_day, 0),
    non_0_users_active_week = nullif (ca.users_active_week, 0),
    non_0_users_active_month = nullif (ca.users_active_month, 0),
    non_0_users_active_half_year = nullif (ca.users_active_half_year, 0),
    non_0_subscribers_local = nullif (ca.subscribers_local, 0),
    non_0_interactions_month = nullif (ca.interactions_month, 0),
    age = CASE WHEN community_is_young THEN
        new_age
    ELSE
        NULL
    END,
    non_0_report_count = nullif (ca.report_count, 0),
    non_0_unresolved_report_count = nullif (ca.unresolved_report_count, 0)
FROM
    community_aggregates AS ca,
    LATERAL (
        SELECT
            extract(minutes FROM (now() - ca.published)) AS new_age),
    LATERAL (
        SELECT
            new_age >= 0
            AND new_age <= 10080 AS community_is_young)
WHERE
    community.id = ca.community_id
    AND (ca.subscribers != 1
        OR ca.posts != 0
        OR ca.comments != 0
        OR ca.users_active_day != 0
        OR ca.users_active_week != 0
        OR ca.users_active_month != 0
        OR ca.users_active_half_year != 0
        OR ca.subscribers_local != 0
        OR ca.interactions_month != 0
        OR community_is_young
        OR ca.report_count != 0
        OR ca.unresolved_report_count != 0);

DROP TABLE community_aggregates;

-- Re-enable triggers after upserts
ALTER TABLE community ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'community');

-- reindex
REINDEX TABLE community;

CREATE INDEX idx_community_hot ON public.community USING btree (get_community_hot_rank (non_1_subscribers, age) DESC);

CREATE INDEX idx_community_young ON public.community USING btree (published)
WHERE (age IS NOT NULL);

CREATE INDEX idx_community_subscribers ON public.community USING btree (coalesce(non_1_subscribers, 1) DESC);

CREATE INDEX idx_community_users_active_month ON public.community USING btree (coalesce(non_0_users_active_month, 0) DESC);

-- merge person_aggregates into person table
ALTER TABLE person
    ADD COLUMN non_0_post_count int,
    ADD COLUMN non_0_post_score int,
    ADD COLUMN non_0_comment_count int,
    ADD COLUMN non_0_comment_score int;

-- Disable the triggers temporarily
ALTER TABLE person DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'person');

UPDATE
    person
SET
    non_0_post_count = nullif (pa.post_count, 0),
    non_0_post_score = nullif (pa.post_score, 0),
    non_0_comment_count = nullif (pa.comment_count, 0),
    non_0_comment_score = nullif (pa.comment_score, 0)
FROM
    person_aggregates AS pa
WHERE
    person.id = pa.person_id
    AND (pa.post_count != 0
        OR pa.post_score != 0
        OR pa.comment_count != 0
        OR pa.comment_score != 0);

DROP TABLE person_aggregates;

-- Re-enable triggers after upserts
ALTER TABLE person ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'person');

-- reindex
REINDEX TABLE person;

-- merge site_aggregates into local_site table
ALTER TABLE local_site
    ADD COLUMN users int NOT NULL DEFAULT 1,
    ADD COLUMN posts int NOT NULL DEFAULT 0,
    ADD COLUMN comments int NOT NULL DEFAULT 0,
    ADD COLUMN communities int NOT NULL DEFAULT 0,
    ADD COLUMN users_active_day int NOT NULL DEFAULT 0,
    ADD COLUMN users_active_week int NOT NULL DEFAULT 0,
    ADD COLUMN users_active_month int NOT NULL DEFAULT 0,
    ADD COLUMN users_active_half_year int NOT NULL DEFAULT 0;

-- Disable the triggers temporarily
ALTER TABLE local_site DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'local_site');

UPDATE
    local_site
SET
    users = sa.users,
    posts = sa.posts,
    comments = sa.comments,
    communities = sa.communities,
    users_active_day = sa.users_active_day,
    users_active_week = sa.users_active_week,
    users_active_month = sa.users_active_month,
    users_active_half_year = sa.users_active_half_year
FROM
    site_aggregates AS sa
WHERE
    local_site.site_id = sa.site_id;

DROP TABLE site_aggregates;

-- Re-enable triggers after upserts
ALTER TABLE local_site ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'local_site');

-- reindex
REINDEX TABLE local_site;

-- merge local_user_vote_display_mode into local_user table
ALTER TABLE local_user
    ADD COLUMN show_score boolean NOT NULL DEFAULT FALSE,
    ADD COLUMN show_upvotes boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN show_downvotes boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN show_upvote_percentage boolean NOT NULL DEFAULT FALSE;

-- Disable the triggers temporarily
ALTER TABLE local_user DISABLE TRIGGER ALL;

-- disable all table indexes
UPDATE
    pg_index
SET
    indisready = FALSE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'local_user');

UPDATE
    local_user
SET
    show_score = v.score,
    show_upvotes = v.upvotes,
    show_downvotes = v.downvotes,
    show_upvote_percentage = v.upvote_percentage
FROM
    local_user_vote_display_mode AS v
WHERE
    local_user.id = v.local_user_id;

DROP TABLE local_user_vote_display_mode;

-- Re-enable triggers after upserts
ALTER TABLE local_user ENABLE TRIGGER ALL;

-- Re-enable indexes
UPDATE
    pg_index
SET
    indisready = TRUE
WHERE
    indrelid = (
        SELECT
            oid
        FROM
            pg_class
        WHERE
            relname = 'local_user');

-- reindex
REINDEX TABLE local_user;

