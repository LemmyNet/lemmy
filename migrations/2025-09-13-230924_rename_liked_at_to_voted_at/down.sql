ALTER TABLE comment_actions RENAME COLUMN voted_at TO liked_at;

ALTER TABLE post_actions RENAME COLUMN voted_at TO liked_at;

