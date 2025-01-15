-- Generate new column last_donation_notification with default value at random time in the
-- past year (so that users dont see it all at the same time after instance upgrade).
ALTER TABLE local_user
    ADD COLUMN last_donation_notification timestamptz NOT NULL DEFAULT (now() - (random() * (interval '12 months')));

ALTER TABLE local_site
    ADD COLUMN disable_donation_dialog boolean NOT NULL DEFAULT FALSE;

