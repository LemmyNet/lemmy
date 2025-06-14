ALTER TABLE person_actions
    ADD COLUMN voted_at timestamptz,
    ADD COLUMN upvotes bigint,
    ADD COLUMN downvotes bigint;

ALTER TABLE local_user
    ADD COLUMN show_person_votes boolean NOT NULL DEFAULT FALSE;

