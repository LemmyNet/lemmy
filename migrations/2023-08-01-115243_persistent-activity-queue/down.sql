ALTER TABLE sent_activity
    DROP COLUMN send_targets,
    DROP COLUMN actor_apub_id,
    DROP COLUMN actor_type;

DROP TYPE actor_type_enum;

DROP TABLE federation_queue_state;

DROP INDEX idx_community_follower_published;

