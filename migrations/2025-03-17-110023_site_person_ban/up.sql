ALTER TABLE instance_actions
    ADD COLUMN received_ban timestamptz;

ALTER TABLE instance_actions
    ADD COLUMN ban_expires timestamptz;

