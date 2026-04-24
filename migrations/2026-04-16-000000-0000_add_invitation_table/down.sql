ALTER TABLE local_site
    DROP COLUMN max_invites_per_user_allowed;

ALTER TABLE local_user
    DROP COLUMN invited_by_local_user_id;

DROP TABLE local_user_invite;

