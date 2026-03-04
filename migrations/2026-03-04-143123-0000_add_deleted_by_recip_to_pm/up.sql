ALTER TABLE private_message
    ADD COLUMN deleted_by_recipient boolean NOT NULL DEFAULT FALSE;

