-- Based on a poll, update the local_user_vote_display_mode defaults to:
-- Upvotes + Downvotes
-- Rather than
-- Score + upvote_percentage
ALTER TABLE local_user_vote_display_mode
    ALTER COLUMN upvotes SET DEFAULT TRUE,
    ALTER COLUMN downvotes SET DEFAULT TRUE,
    ALTER COLUMN score SET DEFAULT FALSE,
    ALTER COLUMN upvote_percentage SET DEFAULT FALSE;

