-- Based on a poll, update the local_user_vote_display_mode defaults to:
-- Upvotes + Downvotes
-- Rather than
-- Score + upvote_percentage
ALTER TABLE local_user_vote_display_mode
    DROP COLUMN score,
    ADD COLUMN score boolean DEFAULT FALSE NOT NULL,
    DROP COLUMN upvotes,
    ADD COLUMN upvotes boolean DEFAULT TRUE NOT NULL,
    DROP COLUMN downvotes,
    ADD COLUMN downvotes boolean DEFAULT TRUE NOT NULL,
    DROP COLUMN upvote_percentage,
    ADD COLUMN upvote_percentage boolean DEFAULT FALSE NOT NULL;

