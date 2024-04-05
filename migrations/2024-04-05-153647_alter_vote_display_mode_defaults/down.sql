ALTER TABLE local_user_vote_display_mode
    ALTER COLUMN upvotes SET DEFAULT FALSE,
    ALTER COLUMN downvotes SET DEFAULT FALSE,
    ALTER COLUMN score SET DEFAULT TRUE,
    ALTER COLUMN upvote_percentage SET DEFAULT TRUE;

