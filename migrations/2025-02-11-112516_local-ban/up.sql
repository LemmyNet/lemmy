-- Separate columns for banning a remote user, this way a ban+unban on home
-- instance wont overwrite local ban
ALTER TABLE person
    ADD COLUMN local_banned boolean NOT NULL DEFAULT FALSE;

ALTER TABLE person
    ADD COLUMN local_ban_expires timestamptz;

-- Same for remote community, local removal should not be overwritten by
-- remove+restore on home instance
ALTER TABLE community
    ADD COLUMN local_removed boolean NOT NULL DEFAULT FALSE;

-- When posting to a remote community mark it as pending until it gets announced back to us.
-- This way the posts of banned users wont appear in the community on other instances.
ALTER TABLE post
    ADD COLUMN pending boolean NOT NULL DEFAULT FALSE;

ALTER TABLE comment
    ADD COLUMN pending boolean NOT NULL DEFAULT FALSE;

