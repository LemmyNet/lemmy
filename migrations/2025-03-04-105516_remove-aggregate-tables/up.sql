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

CREATE INDEX idx_comment_controversy ON comment USING btree (controversy_rank DESC);

CREATE INDEX idx_comment_hot ON comment USING btree (hot_rank DESC, score DESC);

CREATE INDEX idx_comment_nonzero_hotrank ON comment USING btree (published)
WHERE (hot_rank <> (0)::double precision);

--CREATE INDEX idx_comment_published on comment USING btree (published DESC);
CREATE INDEX idx_comment_score ON comment USING btree (score DESC);

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
    ADD COLUMN instance_id int REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE,
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
    ALTER COLUMN instance_id SET NOT NULL,
    ALTER CONSTRAINT post_instance_id_fkey NOT DEFERRABLE;

CREATE INDEX idx_post_community_active ON post USING btree (community_id, featured_local DESC, hot_rank_active DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_controversy ON post USING btree (community_id, featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_community_hot ON post USING btree (community_id, featured_local DESC, hot_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_most_comments ON post USING btree (community_id, featured_local DESC, comments DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time ON post USING btree (community_id, featured_local DESC, newest_comment_time DESC, id DESC);

CREATE INDEX idx_post_community_newest_comment_time_necro ON post USING btree (community_id, featured_local DESC, newest_comment_time_necro DESC, id DESC);

-- INDEX idx_post_community_published ON post USING btree (community_id, featured_local DESC, published DESC);
--CREATE INDEX idx_post_community_published_asc ON post USING btree (community_id, featured_local DESC, reverse_timestamp_sort (published) DESC);
CREATE INDEX idx_post_community_scaled ON post USING btree (community_id, featured_local DESC, scaled_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_community_score ON post USING btree (community_id, featured_local DESC, score DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_active ON post USING btree (community_id, featured_community DESC, hot_rank_active DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_controversy ON post USING btree (community_id, featured_community DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_community_hot ON post USING btree (community_id, featured_community DESC, hot_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_most_comments ON post USING btree (community_id, featured_community DESC, comments DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time ON post USING btree (community_id, featured_community DESC, newest_comment_time DESC, id DESC);

CREATE INDEX idx_post_featured_community_newest_comment_time_necr ON post USING btree (community_id, featured_community DESC, newest_comment_time_necro DESC, id DESC);

--CREATE INDEX idx_post_featured_community_published ON post USING btree (community_id, featured_community DESC, published DESC);
CREATE INDEX idx_post_featured_community_published_asc ON post USING btree (community_id, featured_community DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_community_scaled ON post USING btree (community_id, featured_community DESC, scaled_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_community_score ON post USING btree (community_id, featured_community DESC, score DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_active ON post USING btree (featured_local DESC, hot_rank_active DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_controversy ON post USING btree (featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_local_hot ON post USING btree (featured_local DESC, hot_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_most_comments ON post USING btree (featured_local DESC, comments DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time ON post USING btree (featured_local DESC, newest_comment_time DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_comment_time_necro ON post USING btree (featured_local DESC, newest_comment_time_necro DESC, id DESC);

CREATE INDEX idx_post_featured_local_published ON post USING btree (featured_local DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_published_asc ON post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_scaled ON post USING btree (featured_local DESC, scaled_rank DESC, published DESC, id DESC);

CREATE INDEX idx_post_featured_local_score ON post USING btree (featured_local DESC, score DESC, published DESC, id DESC);

CREATE INDEX idx_post_nonzero_hotrank ON post USING btree (published DESC)
WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

CREATE INDEX idx_post_published ON post USING btree (published DESC);

CREATE INDEX idx_post_published_asc ON post USING btree (reverse_timestamp_sort (published) DESC);

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
    ADD COLUMN interactions_month bigint NOT NULL DEFAULT 0,
    ALTER CONSTRAINT community_instance_id_fkey DEFERRABLE INITIALLY DEFERRED;

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

CREATE INDEX idx_community_hot ON public.community USING btree (hot_rank DESC);

CREATE INDEX idx_community_nonzero_hotrank ON public.community USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_community_subscribers ON public.community USING btree (subscribers DESC);

CREATE INDEX idx_community_users_active_month ON public.community USING btree (users_active_month DESC);

-- merge person_aggregates into person table
ALTER TABLE person
    ADD COLUMN post_count bigint NOT NULL DEFAULT 0,
    ADD COLUMN post_score bigint NOT NULL DEFAULT 0,
    ADD COLUMN comment_count bigint NOT NULL DEFAULT 0,
    ADD COLUMN comment_score bigint NOT NULL DEFAULT 0;

UPDATE
    person
SET
    post_count = pa.post_count,
    post_score = pa.post_score,
    comment_count = pa.comment_count,
    comment_score = pa.comment_score
FROM
    person_aggregates AS pa
WHERE
    person.id = pa.person_id;

-- merge site_aggregates into person table
ALTER TABLE local_site
    ADD COLUMN users bigint NOT NULL DEFAULT 1,
    ADD COLUMN posts bigint NOT NULL DEFAULT 0,
    ADD COLUMN comments bigint NOT NULL DEFAULT 0,
    ADD COLUMN communities bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_day bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_week bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_month bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_half_year bigint NOT NULL DEFAULT 0;

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

-- merge local_user_vote_display_mode into local_user table
ALTER TABLE local_user
    ADD COLUMN show_score boolean NOT NULL DEFAULT FALSE,
    ADD COLUMN show_upvotes boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN show_downvotes boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN show_upvote_percentage boolean NOT NULL DEFAULT FALSE;

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

DROP TABLE comment_aggregates, post_aggregates, community_aggregates, person_aggregates, site_aggregates, local_user_vote_display_mode;

