CREATE TYPE actor_type_enum AS enum(
    'site',
    'community',
    'person'
);

-- actor_apub_id only null for old entries
ALTER TABLE sent_activity
    ADD COLUMN send_targets jsonb NOT NULL DEFAULT '{"inboxes": [], "community_followers_of": [], "all_instances": false}',
    ADD COLUMN actor_type actor_type_enum NOT NULL DEFAULT 'person',
    ADD COLUMN actor_apub_id text DEFAULT NULL;

ALTER TABLE sent_activity
    ALTER COLUMN send_targets DROP DEFAULT,
    ALTER COLUMN actor_type DROP DEFAULT;

CREATE TABLE federation_queue_state(
    domain text PRIMARY KEY,
    last_successful_id bigint NOT NULL,
    fail_count integer NOT NULL,
    last_retry timestamptz NOT NULL
);

-- for incremental fetches of followers
CREATE INDEX idx_community_follower_published ON community_follower(published);

