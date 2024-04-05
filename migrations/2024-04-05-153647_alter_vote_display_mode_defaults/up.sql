-- Based on a poll, update the local_user_vote_display_mode defaults to:
-- Upvotes + Downvotes
-- Rather than
-- Score + upvote_percentage
ALTER TABLE local_user_vote_display_mode
    ALTER COLUMN upvotes SET DEFAULT TRUE,
    ALTER COLUMN downvotes SET DEFAULT TRUE,
    ALTER COLUMN score SET DEFAULT FALSE,
    ALTER COLUMN upvote_percentage SET DEFAULT FALSE;

-- Regenerate the rows with the new default
DELETE FROM local_user_vote_display_mode;

-- Re-insert them
INSERT INTO local_user_vote_display_mode (local_user_id)
SELECT
    id
FROM
    local_user;

