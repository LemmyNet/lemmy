
-- Remove the new columns

alter table post_aggregates drop column hot_rank;
alter table post_aggregates drop column hot_rank_active;

alter table comment_aggregates drop column hot_rank;

alter table community_aggregates drop column hot_rank;

-- Drop some new indexes
drop index idx_post_aggregates_score;
drop index idx_post_aggregates_published;
drop index idx_post_aggregates_newest_comment_time;
drop index idx_post_aggregates_newest_comment_time_necro;
drop index idx_post_aggregates_featured_community;
drop index idx_post_aggregates_featured_local;

-- Recreate the old indexes
CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time DESC);
CREATE INDEX idx_post_aggregates_featured_community_newest_comment_time ON public.post_aggregates USING btree (featured_community DESC, newest_comment_time DESC);
CREATE INDEX idx_post_aggregates_featured_local_comments ON public.post_aggregates USING btree (featured_local DESC, comments DESC);
CREATE INDEX idx_post_aggregates_featured_community_comments ON public.post_aggregates USING btree (featured_community DESC, comments DESC);
CREATE INDEX idx_post_aggregates_featured_local_hot ON public.post_aggregates USING btree (featured_local DESC, hot_rank((score)::numeric, published) DESC, published DESC);
CREATE INDEX idx_post_aggregates_featured_community_hot ON public.post_aggregates USING btree (featured_community DESC, hot_rank((score)::numeric, published) DESC, published DESC);
CREATE INDEX idx_post_aggregates_featured_local_score ON public.post_aggregates USING btree (featured_local DESC, score DESC);
CREATE INDEX idx_post_aggregates_featured_community_score ON public.post_aggregates USING btree (featured_community DESC, score DESC);
CREATE INDEX idx_post_aggregates_featured_local_published ON public.post_aggregates USING btree (featured_local DESC, published DESC);
CREATE INDEX idx_post_aggregates_featured_community_published ON public.post_aggregates USING btree (featured_community DESC, published DESC);
CREATE INDEX idx_post_aggregates_featured_local_active ON public.post_aggregates USING btree (featured_local DESC, hot_rank((score)::numeric, newest_comment_time_necro) DESC, newest_comment_time_necro DESC);
CREATE INDEX idx_post_aggregates_featured_community_active ON public.post_aggregates USING btree (featured_community DESC, hot_rank((score)::numeric, newest_comment_time_necro) DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_comment_aggregates_hot ON public.comment_aggregates USING btree (hot_rank((score)::numeric, published) DESC, published DESC);

CREATE INDEX idx_community_aggregates_hot ON public.community_aggregates USING btree (hot_rank((subscribers)::numeric, published) DESC, published DESC);
