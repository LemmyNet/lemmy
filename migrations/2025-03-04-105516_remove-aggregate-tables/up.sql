-- TODO: update other migrations to handle columns becoming nullable
CREATE FUNCTION get_score (upvotes_nullable integer, downvotes_nullable integer)
    RETURNS integer
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN coalesce (
        upvotes_nullable, 1) - coalesce (
        downvotes_nullable, 0
);

CREATE FUNCTION inner_get_controversy_rank (upvotes integer, downvotes integer)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN downvotes <= 0
        OR upvotes <= 0 THEN
        0
    ELSE
        (
            upvotes + downvotes) ^ CASE WHEN upvotes > downvotes THEN
            downvotes::real / upvotes::real
        ELSE
            upvotes::real / downvotes::real
    END
    END;

CREATE FUNCTION get_controversy_rank (upvotes_nullable integer, downvotes_nullable integer)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN inner_get_controversy_rank (
coalesce(upvotes_nullable, 1), coalesce(downvotes_nullable, 0)
);

CREATE FUNCTION inner_get_hot_rank (score integer, age smallint)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN
    -- after approximately a week (20*32767 seconds, stored in `age` as 32767), hot rank will default to 0.
    CASE WHEN age IS NOT NULL THEN
        -- Use greatest(2,score), so that the hot_rank will be positive and not ignored.
        log (
            greatest (2, score + 2)) / power (((
            -- Age in hours
            age::real / 180) + 2), 1.8)
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        0
    END;

CREATE FUNCTION get_hot_rank (upvotes_nullable integer, downvotes_nullable integer, age smallint)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN inner_get_hot_rank (
        get_score (upvotes_nullable, downvotes_nullable), age
);

CREATE FUNCTION get_scaled_rank (upvotes_nullable integer, downvotes_nullable integer, age smallint, community_interactions_month_nullable integer)
    RETURNS real
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    -- Add 2 to avoid divide by zero errors
    -- Default for score = 1, active users = 1, and now, is (0.1728 / log(2 + 1)) = 0.3621
    -- There may need to be a scale factor multiplied to interactions_month, to make
    -- the log curve less pronounced. This can be tuned in the future.
    RETURN CASE WHEN age IS NOT NULL THEN
        (
            get_hot_rank (upvotes_nullable, downvotes_nullable, age) / log(2 + coalesce(community_interactions_month_nullable, 0)))
    ELSE
        0
    END;

-- Merge comment_aggregates into comment table
ALTER TABLE comment
-- TODO: maybe rename these columns and add getter functions
    ADD COLUMN upvotes int,
    ADD COLUMN downvotes int,
    ADD COLUMN child_count int,
    -- Unlike with other columns, a null value for `age` does not represent one specific default value, but instead indicates that the comment is either supposedly from the future or exceeds the maximum age for hot ranks.
    ADD COLUMN age smallint,
    ADD COLUMN report_count smallint,
    ADD COLUMN unresolved_report_count smallint;

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
    upvotes = nullif (ca.upvotes, 1),
    downvotes = nullif (ca.downvotes, 0),
    child_count = nullif (ca.child_count, 0),
    age = CASE WHEN comment_is_young THEN
        new_age
    ELSE
        NULL
    END,
    report_count = nullif (ca.report_count, 0),
    unresolved_report_count = nullif (ca.unresolved_report_count, 0)
FROM
    comment_aggregates AS ca,
    LATERAL (
        SELECT
            extract(microseconds FROM (now() - ca.published)) / 20000000 AS new_age),
    LATERAL (
        SELECT
            new_age >= 0
            AND new_age <= 20 * 32767 AS comment_is_young)
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
CREATE INDEX idx_comment_controversy ON comment USING btree (get_controversy_rank (upvotes, downvotes) DESC);

CREATE INDEX idx_comment_hot ON comment USING btree (get_hot_rank (upvotes, downvotes, age) DESC, get_score (upvotes, downvotes) DESC);

CREATE INDEX idx_comment_young ON comment USING btree (published)
WHERE (age IS NOT NULL);

--CREATE INDEX idx_comment_published on comment USING btree (published DESC);
CREATE INDEX idx_comment_score ON comment USING btree (get_score (upvotes, downvotes) DESC);

-- merge post_aggregates into post table
ALTER TABLE post
    ADD COLUMN newest_comment_time_necro timestamp with time zone, -- TODO: remove if unused
    ADD COLUMN newest_comment_time timestamp with time zone,
    ADD COLUMN community_interactions_month int,
    ADD COLUMN comments int,
    ADD COLUMN upvotes int,
    ADD COLUMN downvotes int,
    ADD COLUMN age smallint,
    ADD COLUMN newest_non_necro_comment_age smallint,
    ADD COLUMN report_count smallint,
    ADD COLUMN unresolved_report_count smallint;

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
    newest_comment_time_necro = nullif (pa.newest_comment_time_necro, pa.published),
    newest_comment_time = nullif (pa.newest_comment_time, pa.published),
    community_interactions_month = (
        SELECT
            ca.interactions_month
        FROM
            community_aggregates AS ca
        WHERE
            ca.community_id = pa.community_id
            AND post_is_young
            AND ca.interactions_month != 0),
    comments = nullif (pa.comments, 0),
    upvotes = nullif (pa.upvotes, 1),
    downvotes = nullif (pa.downvotes, 0),
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
    report_count = nullif (pa.report_count, 0),
    unresolved_report_count = nullif (pa.unresolved_report_count, 0)
FROM
    post_aggregates AS pa,
    LATERAL (
        SELECT
            extract(microseconds FROM (now() - pa.published)) / 20000000 AS new_age,
            extract(microseconds FROM (now() - pa.newest_comment_time_necro)) / 20000000 AS new_newest_non_necro_comment_age),
    LATERAL (
        SELECT
            new_age >= 0
            AND new_age <= 20 * 32767 AS post_is_young,
            new_newest_non_necro_comment_age >= 0
            AND new_newest_non_necro_comment_age <= 20 * 32767 AS comment_is_young)
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

CREATE INDEX idx_post_community_active ON post USING btree (community_id, featured_local DESC, get_hot_rank (upvotes, downvotes, coalesce(newest_non_necro_comment_age, age)), published DESC, id DESC);

CREATE INDEX idx_post_community_controversy ON post USING btree (community_id, featured_local DESC, get_controversy_rank (upvotes, downvotes) DESC, id DESC);

CREATE INDEX idx_post_community_hot ON post USING btree (community_id, featured_local DESC, get_hot_rank (upvotes, downvotes, age) DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_most_comments ON post USING btree (community_id, featured_local DESC, coalesce(comments, 0) DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time ON post USING btree (community_id, featured_local DESC, coalesce(newest_comment_time, published) DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post USING btree (community_id, featured_local DESC, coalesce(newest_comment_time_necro, published) DESC, id DESC);

-- INDEX idx_post_community_published ON post USING btree (community_id, featured_local DESC, published DESC);
--CREATE INDEX idx_post_community_published_asc ON post USING btree (community_id, featured_local DESC, reverse_timestamp_sort (published) DESC);
CREATE INDEX idx_post_community_scaled ON post USING btree (community_id, featured_local DESC, get_scaled_rank (upvotes, downvotes, age, community_interactions_month) DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_score ON post USING btree (community_id, featured_local DESC, get_score (upvotes, downvotes) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_active ON post USING btree (community_id, featured_community DESC, get_hot_rank (upvotes, downvotes, coalesce(newest_non_necro_comment_age, age)) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_controversy ON post USING btree (community_id, featured_community DESC, get_controversy_rank (upvotes, downvotes) DESC, id DESC);

CREATE INDEX idx_post_featured_community_hot ON post USING btree (community_id, featured_community DESC, get_hot_rank (upvotes, downvotes, age) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_most_comments ON post USING btree (community_id, featured_community DESC, coalesce(comments, 0) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time ON post USING btree (community_id, featured_community DESC, coalesce(newest_comment_time, published) DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time_necr ON post USING btree (community_id, featured_community DESC, coalesce(newest_comment_time_necro, published) DESC, id DESC);

--CREATE INDEX idx_post_featured_community_published ON post USING btree (community_id, featured_community DESC, published DESC);
CREATE INDEX idx_post_featured_community_published_asc ON post USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_community_scaled ON post USING btree (community_id, featured_community DESC, get_scaled_rank (upvotes, downvotes, age, community_interactions_month) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_score ON post USING btree (community_id, featured_community DESC, get_score (upvotes, downvotes) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_active ON post USING btree (featured_local DESC, get_hot_rank (upvotes, downvotes, coalesce(newest_non_necro_comment_age, age)) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_controversy ON post USING btree (featured_local DESC, get_controversy_rank (upvotes, downvotes) DESC, id DESC);

CREATE INDEX idx_post_featured_local_hot ON post USING btree (featured_local DESC, get_hot_rank (upvotes, downvotes, age) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_most_comments ON post USING btree (featured_local DESC, coalesce(comments, 0) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time ON post USING btree (featured_local DESC, coalesce(newest_comment_time, published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time_necro ON post USING btree (featured_local DESC, coalesce(newest_comment_time_necro, published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_published ON post USING btree (featured_local DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_published_asc ON post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_scaled ON post USING btree (featured_local DESC, get_scaled_rank (upvotes, downvotes, age, community_interactions_month) DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_score ON post USING btree (featured_local DESC, get_score (upvotes, downvotes) DESC, published DESC, id DESC);

CREATE INDEX idx_post_young ON post USING btree (published DESC)
WHERE
    age IS NOT NULL OR newest_non_necro_comment_age IS NOT NULL;

CREATE INDEX idx_post_published ON post USING btree (published DESC);

CREATE INDEX idx_post_published_asc ON post USING btree (reverse_timestamp_sort (published) DESC);

-- merge community_aggregates into community table
ALTER TABLE community
    ADD COLUMN subscribers int,
    ADD COLUMN posts int,
    ADD COLUMN comments int,
    ADD COLUMN users_active_day int,
    ADD COLUMN users_active_week int,
    ADD COLUMN users_active_month int,
    ADD COLUMN users_active_half_year int,
    ADD COLUMN subscribers_local int,
    ADD COLUMN interactions_month int,
    ADD COLUMN age smallint,
    ADD COLUMN report_count smallint,
    ADD COLUMN unresolved_report_count smallint;

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
    subscribers = nullif (ca.subscribers, 1),
    posts = nullif (ca.posts, 0),
    comments = nullif (ca.comments, 0),
    users_active_day = nullif (ca.users_active_day, 0),
    users_active_week = nullif (ca.users_active_week, 0),
    users_active_month = nullif (ca.users_active_month, 0),
    users_active_half_year = nullif (ca.users_active_half_year, 0),
    subscribers_local = nullif (ca.subscribers_local, 0),
    interactions_month = nullif (ca.interactions_month, 0),
    age = CASE WHEN community_is_young THEN
        new_age
    ELSE
        NULL
    END,
    report_count = nullif (ca.report_count, 0),
    unresolved_report_count = nullif (ca.unresolved_report_count, 0)
FROM
    community_aggregates AS ca,
    LATERAL (
        SELECT
            extract(microseconds FROM (now() - ca.published)) / 20000000 AS new_age),
    LATERAL (
        SELECT
            new_age >= 0
            AND new_age <= 20 * 32767 AS community_is_young)
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

CREATE INDEX idx_community_hot ON public.community USING btree (inner_get_hot_rank (coalesce(subscribers, 1), age) DESC);

CREATE INDEX idx_community_young ON public.community USING btree (published)
WHERE (age IS NOT NULL);

CREATE INDEX idx_community_subscribers ON public.community USING btree (coalesce(subscribers, 1) DESC);

CREATE INDEX idx_community_users_active_month ON public.community USING btree (coalesce(users_active_month, 0) DESC);

-- merge person_aggregates into person table
ALTER TABLE person
    ADD COLUMN post_count int,
    ADD COLUMN post_score int,
    ADD COLUMN comment_count int,
    ADD COLUMN comment_score int;

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
    post_count = nullif (pa.post_count, 0),
    post_score = nullif (pa.post_score, 0),
    comment_count = nullif (pa.comment_count, 0),
    comment_score = nullif (pa.comment_score, 0)
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

