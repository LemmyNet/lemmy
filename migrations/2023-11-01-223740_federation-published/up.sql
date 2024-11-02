ALTER TABLE federation_queue_state
    ADD COLUMN last_successful_published_time timestamptz NULL,
    ALTER COLUMN last_successful_id DROP NOT NULL,
    ALTER COLUMN last_retry DROP NOT NULL;

