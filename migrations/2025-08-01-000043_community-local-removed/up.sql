-- Same for remote community, local removal should not be overwritten by
-- remove+restore on home instance
ALTER TABLE community
    ADD COLUMN local_removed boolean NOT NULL DEFAULT FALSE;

