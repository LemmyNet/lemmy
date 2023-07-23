CREATE TYPE actor_type_enum AS enum(
    'site',
    'community',
    'person'
);

ALTER TABLE activity
    ADD COLUMN send_targets jsonb DEFAULT NULL,
    ADD COLUMN actor_type actor_type_enum DEFAULT NULL,
    ADD COLUMN actor_apub_id text DEFAULT NULL;

CREATE TABLE federation_queue_state(
    domain text PRIMARY KEY,
    last_successful_id integer NOT NULL,
    fail_count integer NOT NULL,
    last_retry timestamptz NOT NULL
);

-- for incremental fetches of followers
CREATE INDEX idx_community_follower_published ON community_follower(published);

