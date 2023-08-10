CREATE TYPE actor_type_enum AS enum(
    'site',
    'community',
    'person'
);

-- actor_apub_id only null for old entries before this migration
ALTER TABLE sent_activity
    ADD COLUMN send_inboxes text[] NOT NULL DEFAULT '{}', -- list of specific inbox urls
    ADD COLUMN send_community_followers_of integer DEFAULT NULL,
    ADD COLUMN send_all_instances boolean NOT NULL DEFAULT FALSE,
    ADD COLUMN actor_type actor_type_enum NOT NULL DEFAULT 'person',
    ADD COLUMN actor_apub_id text DEFAULT NULL;

ALTER TABLE sent_activity
    ALTER COLUMN send_inboxes DROP DEFAULT,
    ALTER COLUMN send_community_followers_of DROP DEFAULT,
    ALTER COLUMN send_all_instances DROP DEFAULT,
    ALTER COLUMN actor_type DROP DEFAULT,
    ALTER COLUMN actor_apub_id DROP DEFAULT;

CREATE TABLE federation_queue_state(
    id serial PRIMARY KEY,
    domain varchar(255) NOT NULL UNIQUE,
    last_successful_id bigint NOT NULL,
    fail_count integer NOT NULL,
    last_retry timestamptz NOT NULL
);

-- for incremental fetches of followers
CREATE INDEX idx_community_follower_published ON community_follower(published);

