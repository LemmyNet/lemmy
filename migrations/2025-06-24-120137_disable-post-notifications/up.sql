CREATE TYPE notifications_mode_enum AS enum (
    'RepliesAndMentions',
    'All',
    'Mute'
);

ALTER TABLE post_actions
    ADD COLUMN notifications notifications_mode_enum;

