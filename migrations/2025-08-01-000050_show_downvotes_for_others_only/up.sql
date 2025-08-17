-- This changes the local_user.show_downvotes column to an enum,
-- which by default shows all downvotes.
CREATE TYPE vote_show_enum AS ENUM (
    'Show',
    'ShowForOthers',
    'Hide'
);

ALTER TABLE local_user
    ALTER COLUMN show_downvotes DROP DEFAULT;

ALTER TABLE local_user
    ALTER COLUMN show_downvotes TYPE vote_show_enum
    USING
        CASE show_downvotes
        WHEN FALSE THEN
            'Hide'
        ELSE
            'Show'
        END::vote_show_enum;

-- Make ShowForOthers the default
ALTER TABLE local_user
    ALTER COLUMN show_downvotes SET DEFAULT 'Show';

