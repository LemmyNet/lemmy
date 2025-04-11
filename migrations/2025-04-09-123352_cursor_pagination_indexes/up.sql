-- Taglines
CREATE INDEX idx_tagline_published_id ON tagline (published DESC, id DESC);

-- Some for the vote views
CREATE INDEX idx_comment_actions_like_score ON comment_actions (comment_id, like_score, person_id)
WHERE
    like_score IS NOT NULL;

CREATE INDEX idx_post_actions_like_score ON post_actions (post_id, like_score, person_id)
WHERE
    like_score IS NOT NULL;

