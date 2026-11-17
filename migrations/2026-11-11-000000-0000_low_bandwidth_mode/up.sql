-- UI preference only. The backend does not hide media.
-- `Auto` is the default: web UIs treat it as `Never`; mobile apps should follow the device
-- data-saver setting.
CREATE TYPE low_bandwidth_mode_enum AS enum (
    'Auto',
    'Never',
    'Always'
);

ALTER TABLE local_user
    ADD COLUMN low_bandwidth_mode low_bandwidth_mode_enum DEFAULT 'Auto' NOT NULL;

