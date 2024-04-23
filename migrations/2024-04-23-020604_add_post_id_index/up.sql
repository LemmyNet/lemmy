-- Add , post_id DESC to all these
DROP INDEX idx_post_aggregates_community_active;

DROP INDEX idx_post_aggregates_community_controversy;

DROP INDEX idx_post_aggregates_community_hot;

DROP INDEX idx_post_aggregates_community_most_comments;

DROP INDEX idx_post_aggregates_community_newest_comment_time;

DROP INDEX idx_post_aggregates_community_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_community_published;

DROP INDEX idx_post_aggregates_community_published_asc;

DROP INDEX idx_post_aggregates_community_scaled;

DROP INDEX idx_post_aggregates_community_score;

DROP INDEX idx_post_aggregates_featured_community_active;

DROP INDEX idx_post_aggregates_featured_community_controversy;

DROP INDEX idx_post_aggregates_featured_community_hot;

DROP INDEX idx_post_aggregates_featured_community_most_comments;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_community_newest_comment_time_necr;

DROP INDEX idx_post_aggregates_featured_community_published;

DROP INDEX idx_post_aggregates_featured_community_published_asc;

DROP INDEX idx_post_aggregates_featured_community_scaled;

DROP INDEX idx_post_aggregates_featured_community_score;

DROP INDEX idx_post_aggregates_featured_local_active;

DROP INDEX idx_post_aggregates_featured_local_controversy;

DROP INDEX idx_post_aggregates_featured_local_hot;

DROP INDEX idx_post_aggregates_featured_local_most_comments;

DROP INDEX idx_post_aggregates_featured_local_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_local_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_local_published;

DROP INDEX idx_post_aggregates_featured_local_published_asc;

DROP INDEX idx_post_aggregates_featured_local_scaled;

DROP INDEX idx_post_aggregates_featured_local_score;

DROP INDEX idx_post_aggregates_nonzero_hotrank;

DROP INDEX idx_post_aggregates_published;

DROP INDEX idx_post_aggregates_published_asc;

CREATE INDEX idx_post_aggregates_community_active ON public.post_aggregates USING btree (community_id, featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_controversy ON public.post_aggregates USING btree (community_id, featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_hot ON public.post_aggregates USING btree (community_id, featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_most_comments ON public.post_aggregates USING btree (community_id, featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_newest_comment_time ON public.post_aggregates USING btree (community_id, featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_newest_comment_time_necro ON public.post_aggregates USING btree (community_id, featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_published ON public.post_aggregates USING btree (community_id, featured_local DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_published_asc ON public.post_aggregates USING btree (community_id, featured_local DESC, public.reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_scaled ON public.post_aggregates USING btree (community_id, featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_community_score ON public.post_aggregates USING btree (community_id, featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_active ON public.post_aggregates USING btree (community_id, featured_community DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_controversy ON public.post_aggregates USING btree (community_id, featured_community DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_hot ON public.post_aggregates USING btree (community_id, featured_community DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_most_comments ON public.post_aggregates USING btree (community_id, featured_community DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON public.post_aggregates USING btree (community_id, featured_community DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time_necr ON public.post_aggregates USING btree (community_id, featured_community DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_published ON public.post_aggregates USING btree (community_id, featured_community DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_published_asc ON public.post_aggregates USING btree (community_id, featured_community DESC, public.reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_scaled ON public.post_aggregates USING btree (community_id, featured_community DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_community_score ON public.post_aggregates USING btree (community_id, featured_community DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON public.post_aggregates USING btree (featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_controversy ON public.post_aggregates USING btree (featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_hot ON public.post_aggregates USING btree (featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_most_comments ON public.post_aggregates USING btree (featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time_necro ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_published ON public.post_aggregates USING btree (featured_local DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_published_asc ON public.post_aggregates USING btree (featured_local DESC, public.reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_scaled ON public.post_aggregates USING btree (featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON public.post_aggregates USING btree (featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_nonzero_hotrank ON public.post_aggregates USING btree (published DESC)
WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

CREATE INDEX idx_post_aggregates_published ON public.post_aggregates USING btree (published DESC);

CREATE INDEX idx_post_aggregates_published_asc ON public.post_aggregates USING btree (public.reverse_timestamp_sort (published) DESC);

