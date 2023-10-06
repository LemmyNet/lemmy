-- Go through all the tables joins, optimize every view, CTE, etc.
CREATE INDEX idx_post_creator ON post (creator_id);

CREATE INDEX idx_post_community ON post (community_id);

CREATE INDEX idx_post_like_post ON post_like (post_id);

CREATE INDEX idx_post_like_user ON post_like (user_id);

CREATE INDEX idx_comment_creator ON comment (creator_id);

CREATE INDEX idx_comment_parent ON comment (parent_id);

CREATE INDEX idx_comment_post ON comment (post_id);

CREATE INDEX idx_comment_like_comment ON comment_like (comment_id);

CREATE INDEX idx_comment_like_user ON comment_like (user_id);

CREATE INDEX idx_comment_like_post ON comment_like (post_id);

CREATE INDEX idx_community_creator ON community (creator_id);

CREATE INDEX idx_community_category ON community (category_id);

