ALTER TABLE local_user_vote_display_mode
    DROP COLUMN score,
    ADD COLUMN score boolean DEFAULT TRUE NOT NULL,
    DROP COLUMN upvotes,
    ADD COLUMN upvotes boolean DEFAULT FALSE NOT NULL,
    DROP COLUMN downvotes,
    ADD COLUMN downvotes boolean DEFAULT FALSE NOT NULL,
    DROP COLUMN upvote_percentage,
    ADD COLUMN upvote_percentage boolean DEFAULT TRUE NOT NULL;

