-- Add the blur_nsfw to the local user table as a setting
ALTER TABLE local_user
    ADD COLUMN blur_nsfw boolean NOT NULL DEFAULT TRUE;

-- Add the auto_expand to the local user table as a setting
ALTER TABLE local_user
    ADD COLUMN auto_expand boolean NOT NULL DEFAULT FALSE;

