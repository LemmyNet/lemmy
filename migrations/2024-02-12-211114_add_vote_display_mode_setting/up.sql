CREATE TYPE vote_display_mode_enum AS enum (
    'Full',
    'ScoreAndDownvote',
    'ScoreAndUpvotePercentage',
    'UpvotePercentage',
    'Score',
    'HideAll'
);

ALTER TABLE local_user
    ADD COLUMN vote_display_mode vote_display_mode_enum DEFAULT 'ScoreAndUpvotePercentage' NOT NULL;

