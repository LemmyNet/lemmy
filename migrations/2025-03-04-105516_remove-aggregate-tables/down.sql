CREATE FUNCTION old_controversy_rank (upvotes numeric, downvotes numeric)
    RETURNS float
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN CASE WHEN downvotes <= 0
        OR upvotes <= 0 THEN
        0
    ELSE
        (
            upvotes + downvotes) ^ CASE WHEN upvotes > downvotes THEN
            downvotes::float / upvotes::float
        ELSE
            upvotes::float / downvotes::float
    END
    END;

CREATE FUNCTION old_hot_rank (score numeric, published_at timestamp with time zone)
    RETURNS double precision
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE RETURN
    -- after a week, it will default to 0.
    CASE WHEN (
now() - published_at) > '0 days'
        AND (
now() - published_at) < '7 days' THEN
        -- Use greatest(2,score), so that the hot_rank will be positive and not ignored.
        log (
            greatest (2, score + 2)) / power (((EXTRACT(EPOCH FROM (now() - published_at)) / 3600) + 2), 1.8)
    ELSE
        -- if the post is from the future, set hot score to 0. otherwise you can game the post to
        -- always be on top even with only 1 vote by setting it to the future
        0.0
    END;

CREATE FUNCTION old_scaled_rank (score numeric, published_at timestamp with time zone, interactions_month numeric)
    RETURNS double precision
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    -- Add 2 to avoid divide by zero errors
    -- Default for score = 1, active users = 1, and now, is (0.1728 / log(2 + 1)) = 0.3621
    -- There may need to be a scale factor multiplied to interactions_month, to make
    -- the log curve less pronounced. This can be tuned in the future.
    RETURN (
        old_hot_rank (score, published_at) / log(2 + interactions_month)
);

-- move comment_aggregates back into separate table
CREATE TABLE IF NOT EXISTS comment_aggregates (
    comment_id int PRIMARY KEY NOT NULL REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    published timestamp with time zone NOT NULL DEFAULT now(),
    child_count integer NOT NULL DEFAULT 0,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    controversy_rank double precision NOT NULL DEFAULT 0,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0
);

INSERT INTO comment_aggregates
SELECT
    id AS comment_id,
    get_score (non_1_upvotes, non_0_downvotes) AS score,
    coalesce(non_1_upvotes, 1) AS upvotes,
    coalesce(non_0_downvotes, 0) AS downvotes,
    published,
    coalesce(non_0_child_count, 0) AS child_count,
    old_hot_rank (get_score (non_1_upvotes, non_0_downvotes), published) AS hot_rank,
    old_controversy_rank (coalesce(non_1_upvotes, 1), coalesce(non_0_downvotes, 0)) AS controversy_rank,
    coalesce(non_0_report_count, 0) AS report_count,
    coalesce(non_0_unresolved_report_count, 0) AS unresolved_report_count
FROM
    COMMENT
ON CONFLICT (comment_id)
    DO UPDATE SET
        score = excluded.score,
        upvotes = excluded.upvotes,
        downvotes = excluded.downvotes,
        published = excluded.published,
        child_count = excluded.child_count,
        hot_rank = excluded.hot_rank,
        controversy_rank = excluded.controversy_rank,
        report_count = excluded.report_count,
        unresolved_report_count = excluded.unresolved_report_count;

ALTER TABLE comment
    DROP COLUMN non_1_upvotes,
    DROP COLUMN non_0_downvotes,
    DROP COLUMN non_0_child_count,
    DROP COLUMN age,
    DROP COLUMN non_0_report_count,
    DROP COLUMN non_0_unresolved_report_count;

ALTER TABLE comment_aggregates
    ALTER CONSTRAINT comment_aggregates_comment_id_fkey DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX IF NOT EXISTS idx_comment_aggregates_controversy ON comment_aggregates USING btree (controversy_rank DESC);

CREATE INDEX IF NOT EXISTS idx_comment_aggregates_hot ON comment_aggregates USING btree (hot_rank DESC, score DESC);

CREATE INDEX IF NOT EXISTS idx_comment_aggregates_nonzero_hotrank ON comment_aggregates USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX IF NOT EXISTS idx_comment_aggregates_published ON comment_aggregates USING btree (published DESC);

CREATE INDEX IF NOT EXISTS idx_comment_aggregates_score ON comment_aggregates USING btree (score DESC);

-- move comment_aggregates back into separate table
CREATE TABLE IF NOT EXISTS post_aggregates (
    post_id int PRIMARY KEY NOT NULL REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    comments bigint NOT NULL DEFAULT 0,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    published timestamp with time zone NOT NULL DEFAULT now(),
    newest_comment_time_necro timestamp with time zone NOT NULL DEFAULT now(),
    newest_comment_time timestamp with time zone NOT NULL DEFAULT now(),
    featured_community boolean NOT NULL DEFAULT FALSE,
    featured_local boolean NOT NULL DEFAULT FALSE,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    hot_rank_active double precision NOT NULL DEFAULT 0.0001,
    community_id integer NOT NULL REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE,
    creator_id integer NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE,
    controversy_rank double precision NOT NULL DEFAULT 0,
    instance_id integer NOT NULL REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE,
    scaled_rank double precision NOT NULL DEFAULT 0.0001,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0
);

INSERT INTO post_aggregates
SELECT
    id AS post_id,
    coalesce(non_0_comments, 0) AS comments,
    get_score (non_1_upvotes, non_0_downvotes) AS score,
    coalesce(non_1_upvotes, 1) AS upvotes,
    coalesce(non_0_downvotes, 0) AS downvotes,
    published,
    coalesce(newest_comment_time_necro_after_published, published) AS newest_comment_time_necro,
    coalesce(newest_comment_time_after_published, published) AS newest_comment_time,
    featured_community,
    featured_local,
    old_hot_rank (get_score (non_1_upvotes, non_0_downvotes), published) AS hot_rank,
    old_hot_rank (get_score (non_1_upvotes, non_0_downvotes), coalesce(newest_comment_time_necro_after_published, published)) AS hot_rank_active,
    community_id,
    creator_id,
    old_controversy_rank (coalesce(non_1_upvotes, 1), coalesce(non_0_downvotes, 0)) AS controversy_rank,
    (
        SELECT
            community.instance_id
        FROM
            community
        WHERE
            community.id = post.community_id) AS instance_id,
    old_scaled_rank (get_score (non_1_upvotes, non_0_downvotes), published, coalesce(non_0_community_interactions_month, 0)) AS scaled_rank,
    coalesce(non_0_report_count, 0) AS report_count,
    coalesce(non_0_unresolved_report_count, 0) AS unresolved_report_count
FROM
    post
ON CONFLICT (post_id)
    DO UPDATE SET
        comments = excluded.comments,
        score = excluded.score,
        upvotes = excluded.upvotes,
        downvotes = excluded.downvotes,
        published = excluded.published,
        newest_comment_time_necro = excluded.newest_comment_time_necro,
        newest_comment_time = excluded.newest_comment_time,
        featured_community = excluded.featured_community,
        featured_local = excluded.featured_local,
        hot_rank = excluded.hot_rank,
        hot_rank_active = excluded.hot_rank_active,
        community_id = excluded.community_id,
        creator_id = excluded.creator_id,
        controversy_rank = excluded.controversy_rank,
        instance_id = excluded.instance_id,
        scaled_rank = excluded.scaled_rank,
        report_count = excluded.report_count,
        unresolved_report_count = excluded.unresolved_report_count;

ALTER TABLE post
    DROP COLUMN newest_comment_time_necro_after_published,
    DROP COLUMN newest_comment_time_after_published,
    DROP COLUMN non_0_community_interactions_month,
    DROP COLUMN non_0_comments,
    DROP COLUMN non_1_upvotes,
    DROP COLUMN non_0_downvotes,
    DROP COLUMN age,
    DROP COLUMN newest_non_necro_comment_age,
    DROP COLUMN non_0_report_count,
    DROP COLUMN non_0_unresolved_report_count;

ALTER TABLE post_aggregates
    ALTER CONSTRAINT post_aggregates_community_id_fkey DEFERRABLE INITIALLY DEFERRED,
    ALTER CONSTRAINT post_aggregates_creator_id_fkey DEFERRABLE INITIALLY DEFERRED,
    ALTER CONSTRAINT post_aggregates_instance_id_fkey DEFERRABLE INITIALLY DEFERRED,
    ALTER CONSTRAINT post_aggregates_post_id_fkey DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_active ON post_aggregates USING btree (community_id, featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_controversy ON post_aggregates USING btree (community_id, featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_hot ON post_aggregates USING btree (community_id, featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_most_comments ON post_aggregates USING btree (community_id, featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_newest_comment_time ON post_aggregates USING btree (community_id, featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_newest_comment_time_necro ON post_aggregates USING btree (community_id, featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_published ON post_aggregates USING btree (community_id, featured_local DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_published_asc ON post_aggregates USING btree (community_id, featured_local DESC, reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_scaled ON post_aggregates USING btree (community_id, featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_community_score ON post_aggregates USING btree (community_id, featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_active ON post_aggregates USING btree (community_id, featured_community DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_controversy ON post_aggregates USING btree (community_id, featured_community DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_hot ON post_aggregates USING btree (community_id, featured_community DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_most_comments ON post_aggregates USING btree (community_id, featured_community DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_newest_comment_time ON post_aggregates USING btree (community_id, featured_community DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_newest_comment_time_necr ON post_aggregates USING btree (community_id, featured_community DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_published ON post_aggregates USING btree (community_id, featured_community DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_published_asc ON post_aggregates USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_scaled ON post_aggregates USING btree (community_id, featured_community DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_community_score ON post_aggregates USING btree (community_id, featured_community DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_active ON post_aggregates USING btree (featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_controversy ON post_aggregates USING btree (featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_hot ON post_aggregates USING btree (featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_most_comments ON post_aggregates USING btree (featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_newest_comment_time ON post_aggregates USING btree (featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_newest_comment_time_necro ON post_aggregates USING btree (featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_published ON post_aggregates USING btree (featured_local DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_published_asc ON post_aggregates USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_scaled ON post_aggregates USING btree (featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_featured_local_score ON post_aggregates USING btree (featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_nonzero_hotrank ON post_aggregates USING btree (published DESC)
WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

CREATE INDEX IF NOT EXISTS idx_post_aggregates_published ON post_aggregates USING btree (published DESC);

CREATE INDEX IF NOT EXISTS idx_post_aggregates_published_asc ON post_aggregates USING btree (reverse_timestamp_sort (published) DESC);

DROP INDEX idx_post_featured_community_published_asc;

DROP INDEX idx_post_featured_local_published;

DROP INDEX idx_post_featured_local_published_asc;

DROP INDEX idx_post_published;

DROP INDEX idx_post_published_asc;

DROP INDEX idx_search_combined_score;

-- move community_aggregates back into separate table
CREATE TABLE community_aggregates (
    community_id int PRIMARY KEY NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    subscribers bigint NOT NULL DEFAULT 0,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    published timestamp with time zone DEFAULT now() NOT NULL,
    users_active_day bigint NOT NULL DEFAULT 0,
    users_active_week bigint NOT NULL DEFAULT 0,
    users_active_month bigint NOT NULL DEFAULT 0,
    users_active_half_year bigint NOT NULL DEFAULT 0,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    subscribers_local bigint NOT NULL DEFAULT 0,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0,
    interactions_month bigint NOT NULL DEFAULT 0
);

INSERT INTO community_aggregates
SELECT
    id AS comment_id,
    coalesce(non_1_subscribers, 1) AS subscribers,
    coalesce(non_0_posts, 0) AS posts,
    coalesce(non_0_comments, 0) AS comments,
    published,
    coalesce(non_0_users_active_day, 0) AS users_active_day,
    coalesce(non_0_users_active_week, 0) AS users_active_week,
    coalesce(non_0_users_active_month, 0) AS users_active_month,
    coalesce(non_0_users_active_half_year, 0) AS users_active_half_year,
    old_hot_rank (coalesce(non_1_subscribers, 1), published) AS hot_rank,
    coalesce(non_0_subscribers_local, 0) AS subscribers_local,
    coalesce(non_0_report_count, 0) AS report_count,
    coalesce(non_0_unresolved_report_count, 0) AS unresolved_report_count,
    coalesce(non_0_interactions_month, 0) AS interactions_month
FROM
    community;

ALTER TABLE community
    DROP COLUMN non_1_subscribers,
    DROP COLUMN non_0_posts,
    DROP COLUMN non_0_comments,
    DROP COLUMN non_0_users_active_day,
    DROP COLUMN non_0_users_active_week,
    DROP COLUMN non_0_users_active_month,
    DROP COLUMN non_0_users_active_half_year,
    DROP COLUMN non_0_subscribers_local,
    DROP COLUMN non_0_interactions_month,
    DROP COLUMN age,
    DROP COLUMN non_0_report_count,
    DROP COLUMN non_0_unresolved_report_count;

ALTER TABLE community
    ALTER CONSTRAINT community_instance_id_fkey NOT DEFERRABLE;

CREATE INDEX idx_community_aggregates_hot ON public.community_aggregates USING btree (hot_rank DESC);

CREATE INDEX idx_community_aggregates_nonzero_hotrank ON public.community_aggregates USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_community_aggregates_published ON public.community_aggregates USING btree (published DESC);

CREATE INDEX idx_community_aggregates_subscribers ON public.community_aggregates USING btree (subscribers DESC);

CREATE INDEX idx_community_aggregates_users_active_month ON public.community_aggregates USING btree (users_active_month DESC);

-- move person_aggregates back into separate table
CREATE TABLE person_aggregates (
    person_id int PRIMARY KEY NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    post_count bigint NOT NULL DEFAULT 0,
    post_score bigint NOT NULL DEFAULT 0,
    comment_count bigint NOT NULL DEFAULT 0,
    comment_score bigint NOT NULL DEFAULT 0,
    published timestamp with time zone DEFAULT now() NOT NULL
);

INSERT INTO person_aggregates
SELECT
    id AS person_id,
    coalesce(non_0_post_count, 0) AS post_count,
    coalesce(non_0_post_score, 0) AS post_score,
    coalesce(non_0_comment_count, 0) AS comment_count,
    coalesce(non_0_comment_score, 0) AS comment_score,
    published
FROM
    person;

ALTER TABLE person
    DROP COLUMN non_0_post_count,
    DROP COLUMN non_0_post_score,
    DROP COLUMN non_0_comment_count,
    DROP COLUMN non_0_comment_score;

ALTER TABLE person_aggregates
    ALTER CONSTRAINT person_aggregates_person_id_fkey DEFERRABLE INITIALLY DEFERRED;

CREATE INDEX idx_person_aggregates_comment_score ON public.person_aggregates USING btree (comment_score DESC);

CREATE INDEX idx_person_aggregates_person ON public.person_aggregates USING btree (person_id);

-- move site_aggregates back into separate table
CREATE TABLE site_aggregates (
    site_id int PRIMARY KEY NOT NULL REFERENCES site ON UPDATE CASCADE ON DELETE CASCADE,
    users bigint NOT NULL DEFAULT 1,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    communities bigint NOT NULL DEFAULT 0,
    users_active_day bigint NOT NULL DEFAULT 0,
    users_active_week bigint NOT NULL DEFAULT 0,
    users_active_month bigint NOT NULL DEFAULT 0,
    users_active_half_year bigint NOT NULL DEFAULT 0
);

INSERT INTO site_aggregates
SELECT
    id AS site_id,
    users,
    posts,
    comments,
    communities,
    users_active_day,
    users_active_week,
    users_active_month,
    users_active_half_year
FROM
    local_site;

ALTER TABLE local_site
    DROP COLUMN users,
    DROP COLUMN posts,
    DROP COLUMN comments,
    DROP COLUMN communities,
    DROP COLUMN users_active_day,
    DROP COLUMN users_active_week,
    DROP COLUMN users_active_month,
    DROP COLUMN users_active_half_year;

-- move local_user_vote_display_mode back into separate table
CREATE TABLE local_user_vote_display_mode (
    local_user_id int PRIMARY KEY NOT NULL REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE,
    score boolean NOT NULL DEFAULT FALSE,
    upvotes boolean NOT NULL DEFAULT TRUE,
    downvotes boolean NOT NULL DEFAULT TRUE,
    upvote_percentage boolean NOT NULL DEFAULT FALSE
);

INSERT INTO local_user_vote_display_mode
SELECT
    id AS local_user_id,
    show_score AS score,
    show_upvotes AS upvotes,
    show_downvotes AS downvotes,
    show_upvote_percentage AS upvote_percentage
FROM
    local_user;

ALTER TABLE local_user
    DROP COLUMN show_score,
    DROP COLUMN show_upvotes,
    DROP COLUMN show_downvotes,
    DROP COLUMN show_upvote_percentage;

CREATE INDEX idx_search_combined_score ON public.search_combined USING btree (coalesce(non_1_score, 1) DESC, id DESC);

ALTER TABLE site_aggregates
    ALTER CONSTRAINT site_aggregates_site_id_fkey DEFERRABLE INITIALLY DEFERRED;

CREATE UNIQUE INDEX idx_site_aggregates_1_row_only ON public.site_aggregates USING btree ((TRUE));

ALTER TABLE community_aggregates
    ALTER CONSTRAINT community_aggregates_community_id_fkey DEFERRABLE INITIALLY DEFERRED;

DROP FUNCTION age_of, get_community_hot_rank, get_controversy_rank, get_hot_rank, get_scaled_rank, get_score, inner_age, inner_get_hot_rank, old_controversy_rank, old_hot_rank, old_scaled_rank;

