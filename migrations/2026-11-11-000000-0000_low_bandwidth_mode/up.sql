-- UI preference only. The backend does not hide media. `auto` stays a client decision.
CREATE TYPE low_bandwidth_mode_enum AS enum (
    'Never',
    'Always'
);

ALTER TABLE local_user
    ADD COLUMN low_bandwidth_mode low_bandwidth_mode_enum DEFAULT 'Never' NOT NULL;

