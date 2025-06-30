CREATE TYPE post_notifications_mode_enum AS enum (
    'RepliesAndMentions',
    'AllComments',
    'Mute'
);

ALTER TABLE post_actions
    ADD COLUMN notifications post_notifications_mode_enum;

