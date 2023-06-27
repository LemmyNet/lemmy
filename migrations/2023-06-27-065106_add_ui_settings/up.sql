
-- Add the blur_nsfw to the local user table as a setting
alter table local_user add column blur_nsfw boolean not null default true;

-- Add the auto_expand to the local user table as a setting
alter table local_user add column auto_expand boolean not null default false;
