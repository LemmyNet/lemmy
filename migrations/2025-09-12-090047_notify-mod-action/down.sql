ALTER TABLE notification
    DROP COLUMN mod_remove_comment_id,
    DROP COLUMN admin_add_id,
    DROP COLUMN mod_add_to_community_id,
    DROP COLUMN admin_ban_id,
    DROP COLUMN mod_ban_from_community_id,
    DROP COLUMN mod_lock_post_id,
    DROP COLUMN admin_remove_community_id,
    DROP COLUMN mod_remove_post_id,
    DROP COLUMN mod_lock_comment_id;

-- rename the old enum
ALTER TYPE notification_type_enum RENAME TO notification_type_enum__;

DELETE FROM notification
WHERE kind = 'ModAction'
    OR kind = 'RevertModAction';

-- create the new enum
CREATE TYPE notification_type_enum AS ENUM (
    'Mention',
    'Reply',
    'Subscribed',
    'PrivateMessage'
);

-- alter all your enum columns
ALTER TABLE notification
    ALTER COLUMN kind TYPE notification_type_enum
    USING kind::text::notification_type_enum;

-- drop the old enum
DROP TYPE notification_type_enum__;

