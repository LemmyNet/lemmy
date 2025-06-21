ALTER TABLE person_actions
    DROP COLUMN voted_at,
    DROP COLUMN upvotes,
    DROP COLUMN downvotes;

ALTER TABLE local_user
    DROP COLUMN show_person_votes;

