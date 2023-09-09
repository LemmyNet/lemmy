ALTER TABLE sent_activity
    DROP COLUMN send_inboxes,
    DROP COLUMN send_community_followers_of,
    DROP COLUMN send_all_instances,
    DROP COLUMN actor_apub_id,
    DROP COLUMN actor_type;

DROP TYPE actor_type_enum;

DROP TABLE federation_queue_state;

DROP INDEX idx_community_follower_published;

