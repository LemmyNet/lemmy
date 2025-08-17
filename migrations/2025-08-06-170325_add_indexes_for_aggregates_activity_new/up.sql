CREATE INDEX idx_post_actions_liked_at ON post_actions (liked_at)
WHERE
    liked_at IS NOT NULL;

CREATE INDEX idx_comment_actions_liked_at ON comment_actions (liked_at)
WHERE
    liked_at IS NOT NULL;

