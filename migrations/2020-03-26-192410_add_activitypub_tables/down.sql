DROP TABLE activity;

ALTER TABLE user_
    DROP COLUMN actor_id,
    DROP COLUMN private_key,
    DROP COLUMN public_key,
    DROP COLUMN bio,
    DROP COLUMN local,
    DROP COLUMN last_refreshed_at;

ALTER TABLE community
    DROP COLUMN actor_id,
    DROP COLUMN private_key,
    DROP COLUMN public_key,
    DROP COLUMN local,
    DROP COLUMN last_refreshed_at;

