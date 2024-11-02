ALTER TABLE federation_queue_state
    DROP COLUMN last_successful_published_time,
    ALTER COLUMN last_successful_id SET NOT NULL,
    ALTER COLUMN last_retry SET NOT NULL;

