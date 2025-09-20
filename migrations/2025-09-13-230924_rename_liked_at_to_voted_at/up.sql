ALTER TABLE comment_actions RENAME COLUMN liked_at TO voted_at;

ALTER TABLE post_actions RENAME COLUMN liked_at TO voted_at;

