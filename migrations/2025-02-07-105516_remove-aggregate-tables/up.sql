-- merge comment_aggregates into comment table
ALTER TABLE comment
    ADD COLUMN score bigint NOT NULL DEFAULT 0,
    ADD COLUMN upvotes bigint NOT NULL DEFAULT 0,
    ADD COLUMN downvotes bigint NOT NULL DEFAULT 0,
    ADD COLUMN child_count integer NOT NULL DEFAULT 0,
    ADD COLUMN hot_rank double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN controversy_rank double precision NOT NULL DEFAULT 0,
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

UPDATE
    comment
SET
    score = ca.score,
    upvotes = ca.upvotes,
    downvotes = ca.downvotes,
    child_count = ca.child_count,
    hot_rank = ca.hot_rank,
    controversy_rank = ca.controversy_rank,
    report_count = ca.report_count,
    unresolved_report_count = ca.unresolved_report_count
FROM
    comment_aggregates AS ca
WHERE
    comment.id = ca.comment_id;

DROP TABLE comment_aggregates;

CREATE INDEX idx_comment_aggregates_controversy ON comment USING btree (controversy_rank DESC);

CREATE INDEX idx_comment_aggregates_hot ON comment USING btree (hot_rank DESC, score DESC);

CREATE INDEX idx_comment_aggregates_nonzero_hotrank ON comment USING btree (published)
WHERE (hot_rank <> (0)::double precision);

-- some indexes commented out because they already exist
--CREATE INDEX idx_comment_aggregates_published on comment USING btree (published DESC);
CREATE INDEX idx_comment_aggregates_score ON comment USING btree (score DESC);

-- merge post_aggregates into post table
ALTER TABLE post
    ADD COLUMN comments bigint NOT NULL DEFAULT 0,
    ADD COLUMN score bigint NOT NULL DEFAULT 0,
    ADD COLUMN upvotes bigint NOT NULL DEFAULT 0,
    ADD COLUMN downvotes bigint NOT NULL DEFAULT 0,
    ADD COLUMN newest_comment_time_necro timestamp with time zone NOT NULL DEFAULT now(),
    ADD COLUMN newest_comment_time timestamp with time zone NOT NULL DEFAULT now(),
    ADD COLUMN hot_rank double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN hot_rank_active double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN controversy_rank double precision NOT NULL DEFAULT 0,
    ADD COLUMN instance_id int NOT NULL DEFAULT 0 REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    ADD COLUMN scaled_rank double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

UPDATE
    post
SET
    comments = pa.comments,
    score = pa.score,
    upvotes = pa.upvotes,
    downvotes = pa.downvotes,
    newest_comment_time_necro = pa.newest_comment_time_necro,
    newest_comment_time = pa.newest_comment_time,
    hot_rank = pa.hot_rank,
    hot_rank_active = pa.hot_rank_active,
    controversy_rank = pa.controversy_rank,
    instance_id = pa.instance_id,
    scaled_rank = pa.scaled_rank,
    report_count = pa.report_count,
    unresolved_report_count = pa.unresolved_report_count
FROM
    post_aggregates AS pa
WHERE
    post.id = pa.post_id;

ALTER TABLE post
    ALTER COLUMN instance_id DROP NOT NULL;

DROP TABLE post_aggregates;

-- Note, removed `post_id DESC` from all these
CREATE INDEX idx_post_aggregates_community_active ON post USING btree (community_id, featured_local DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_controversy ON post USING btree (community_id, featured_local DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_community_hot ON post USING btree (community_id, featured_local DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_most_comments ON post USING btree (community_id, featured_local DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_newest_comment_time ON post USING btree (community_id, featured_local DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_community_newest_comment_time_necro ON post USING btree (community_id, featured_local DESC, newest_comment_time_necro DESC);

--CREATE INDEX idx_post_aggregates_community_published on post USING btree (community_id, featured_local DESC, published DESC);
--CREATE INDEX idx_post_aggregates_community_published_asc on post USING btree (community_id, featured_local DESC, reverse_timestamp_sort (published) DESC);
CREATE INDEX idx_post_aggregates_community_scaled ON post USING btree (community_id, featured_local DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_community_score ON post USING btree (community_id, featured_local DESC, score DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_active ON post USING btree (community_id, featured_community DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_controversy ON post USING btree (community_id, featured_community DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_community_hot ON post USING btree (community_id, featured_community DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_most_comments ON post USING btree (community_id, featured_community DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON post USING btree (community_id, featured_community DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time_necr ON post USING btree (community_id, featured_community DESC, newest_comment_time_necro DESC);

--CREATE INDEX idx_post_aggregates_featured_community_published on post USING btree (community_id, featured_community DESC, published DESC);
--CREATE INDEX idx_post_aggregates_featured_community_published_asc on post USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC);
CREATE INDEX idx_post_aggregates_featured_community_scaled ON post USING btree (community_id, featured_community DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_community_score ON post USING btree (community_id, featured_community DESC, score DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON post USING btree (featured_local DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_controversy ON post USING btree (featured_local DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_local_hot ON post USING btree (featured_local DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_most_comments ON post USING btree (featured_local DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON post USING btree (featured_local DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time_necro ON post USING btree (featured_local DESC, newest_comment_time_necro DESC);

--CREATE INDEX idx_post_aggregates_featured_local_published on post USING btree (featured_local DESC, published DESC);
--CREATE INDEX idx_post_aggregates_featured_local_published_asc on post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC);
CREATE INDEX idx_post_aggregates_featured_local_scaled ON post USING btree (featured_local DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON post USING btree (featured_local DESC, score DESC, published DESC);

CREATE INDEX idx_post_aggregates_nonzero_hotrank ON post USING btree (published DESC)
WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

--CREATE INDEX idx_post_aggregates_published on post USING btree (published DESC);
--CREATE INDEX idx_post_aggregates_published_asc on post USING btree (reverse_timestamp_sort (published) DESC);
-- merge community_aggregates into community table
ALTER TABLE community
    ADD COLUMN subscribers bigint NOT NULL DEFAULT 0,
    ADD COLUMN posts bigint NOT NULL DEFAULT 0,
    ADD COLUMN comments bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_day bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_week bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_month bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_half_year bigint NOT NULL DEFAULT 0,
    ADD COLUMN hot_rank double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN subscribers_local bigint NOT NULL DEFAULT 0,
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN interactions_month bigint NOT NULL DEFAULT 0;

UPDATE
    community
SET
    subscribers = ca.subscribers,
    posts = ca.posts,
    comments = ca.comments,
    users_active_day = ca.users_active_day,
    users_active_week = ca.users_active_week,
    users_active_month = ca.users_active_month,
    users_active_half_year = ca.users_active_half_year,
    hot_rank = ca.hot_rank,
    subscribers_local = ca.subscribers_local,
    report_count = ca.report_count,
    unresolved_report_count = ca.unresolved_report_count,
    interactions_month = ca.interactions_month
FROM
    community_aggregates AS ca
WHERE
    community.id = ca.community_id;

DROP TABLE community_aggregates;

CREATE INDEX idx_community_aggregates_hot ON public.community USING btree (hot_rank DESC);

CREATE INDEX idx_community_aggregates_nonzero_hotrank ON public.community USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_community_aggregates_subscribers ON public.community USING btree (subscribers DESC);

CREATE INDEX idx_community_aggregates_users_active_month ON public.community USING btree (users_active_month DESC);

