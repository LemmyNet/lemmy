CREATE INDEX idx_post_actions_voted_at ON post_actions (voted_at)
WHERE
    voted_at IS NOT NULL;

CREATE INDEX idx_comment_actions_voted_at ON comment_actions (voted_at)
WHERE
    voted_at IS NOT NULL;

