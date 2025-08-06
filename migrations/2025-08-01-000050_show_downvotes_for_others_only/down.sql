ALTER TABLE local_user
    ALTER COLUMN show_downvotes DROP DEFAULT;

ALTER TABLE local_user
    ALTER COLUMN show_downvotes TYPE boolean
    USING
        CASE show_downvotes
        WHEN 'Hide' THEN
            FALSE
        ELSE
            TRUE
        END;

-- Make true the default
ALTER TABLE local_user
    ALTER COLUMN show_downvotes SET DEFAULT TRUE;

DROP TYPE vote_show_enum;

