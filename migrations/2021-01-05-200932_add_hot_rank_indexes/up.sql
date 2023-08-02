-- Need to add immutable to the hot_rank function in order to index by it
-- Rank = ScaleFactor * sign(Score) * log(1 + abs(Score)) / (Time + 2)^Gravity
CREATE OR REPLACE FUNCTION hot_rank (score numeric, published timestamp without time zone)
    RETURNS integer
    AS $$
BEGIN
    -- hours_diff:=EXTRACT(EPOCH FROM (timezone('utc',now()) - published))/3600
    RETURN floor(10000 * log(greatest (1, score + 3)) / power(((EXTRACT(EPOCH FROM (timezone('utc', now()) - published)) / 3600) + 2), 1.8))::integer;
END;
$$
LANGUAGE plpgsql
IMMUTABLE;

-- Post_aggregates
CREATE INDEX idx_post_aggregates_stickied_hot ON post_aggregates (stickied DESC, hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_post_aggregates_hot ON post_aggregates (hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_post_aggregates_stickied_active ON post_aggregates (stickied DESC, hot_rank (score, newest_comment_time) DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_active ON post_aggregates (hot_rank (score, newest_comment_time) DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_stickied_score ON post_aggregates (stickied DESC, score DESC);

CREATE INDEX idx_post_aggregates_score ON post_aggregates (score DESC);

CREATE INDEX idx_post_aggregates_stickied_published ON post_aggregates (stickied DESC, published DESC);

CREATE INDEX idx_post_aggregates_published ON post_aggregates (published DESC);

-- Comment
CREATE INDEX idx_comment_published ON comment (published DESC);

-- Comment_aggregates
CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_comment_aggregates_score ON comment_aggregates (score DESC);

-- User
CREATE INDEX idx_user_published ON user_ (published DESC);

-- User_aggregates
CREATE INDEX idx_user_aggregates_comment_score ON user_aggregates (comment_score DESC);

-- Community
CREATE INDEX idx_community_published ON community (published DESC);

-- Community_aggregates
CREATE INDEX idx_community_aggregates_hot ON community_aggregates (hot_rank (subscribers, published) DESC, published DESC);

CREATE INDEX idx_community_aggregates_subscribers ON community_aggregates (subscribers DESC);

