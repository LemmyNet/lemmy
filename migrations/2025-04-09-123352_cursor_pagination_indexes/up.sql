-- Taglines
CREATE INDEX idx_tagline_published_id on tagline (published desc, id desc);

-- Some for the vote views
CREATE INDEX idx_comment_actions_like_score on comment_actions (comment_id, like_score, person_id) where like_score is not null;
CREATE INDEX idx_post_actions_like_score on post_actions (post_id, like_score, person_id) where like_score is not null;
