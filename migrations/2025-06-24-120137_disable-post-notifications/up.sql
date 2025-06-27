CREATE TYPE post_notifications_mode_enum AS enum (
    'replies_and_mentions',
    'all_comments',
    'mute'
);

ALTER TABLE post_actions
    ADD COLUMN notifications post_notifications_mode_enum;

