ALTER TABLE site
    ADD COLUMN actor_id varchar(255) NOT NULL UNIQUE DEFAULT generate_unique_changeme (),
    ADD COLUMN last_refreshed_at Timestamp NOT NULL DEFAULT now(),
    ADD COLUMN inbox_url varchar(255) NOT NULL DEFAULT generate_unique_changeme (),
    ADD COLUMN private_key text,
    ADD COLUMN public_key text NOT NULL DEFAULT generate_unique_changeme ();

