ALTER TABLE mod_ban
    DROP COLUMN instance_id;

ALTER TABLE person
    ADD COLUMN banned boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN published_new timestamp with time zone DEFAULT now() NOT NULL,
    ADD COLUMN updated_new timestamp with time zone,
    ADD COLUMN ap_id_new varchar(255) DEFAULT generate_unique_changeme () NOT NULL,
    ADD COLUMN bio_new text,
    ADD COLUMN local_new boolean NOT NULL DEFAULT TRUE,
    ADD COLUMN private_key_new text,
    ADD COLUMN public_key_new text,
    ADD COLUMN last_refreshed_at_new timestamptz DEFAULT now() NOT NULL,
    ADD COLUMN banner_new text,
    ADD COLUMN deleted_new boolean NOT NULL DEFAULT FALSE,
    ADD COLUMN inbox_url_new varchar(255) DEFAULT generate_unique_changeme () NOT NULL,
    ADD COLUMN matrix_user_id_new text,
    ADD COLUMN bot_account_new boolean DEFAULT FALSE NOT NULL,
    ADD COLUMN ban_expires timestamptz,
    ADD COLUMN instance_id_new int,
    ADD COLUMN post_count_new bigint DEFAULT 0 NOT NULL,
    ADD COLUMN post_score_new bigint DEFAULT 0 NOT NULL,
    ADD COLUMN comment_count_new bigint DEFAULT 0 NOT NULL,
    ADD COLUMN comment_score_new bigint DEFAULT 0 NOT NULL;

UPDATE
    person
SET
    (published_new,
        updated_new,
        ap_id_new,
        bio_new,
        local_new,
        private_key_new,
        public_key_new,
        last_refreshed_at_new,
        banner_new,
        deleted_new,
        inbox_url_new,
        matrix_user_id_new,
        bot_account_new,
        instance_id_new,
        post_count_new,
        post_score_new,
        comment_count_new,
        comment_score_new) = (published,
        updated,
        ap_id,
        bio,
        local,
        private_key,
        public_key,
        last_refreshed_at,
        banner,
        deleted,
        inbox_url,
        matrix_user_id,
        bot_account,
        instance_id,
        post_count,
        post_score,
        comment_count,
        comment_score);

ALTER TABLE person
    DROP COLUMN published,
    DROP COLUMN updated,
    DROP COLUMN ap_id,
    DROP COLUMN bio,
    DROP COLUMN local,
    DROP COLUMN private_key,
    DROP COLUMN public_key,
    DROP COLUMN last_refreshed_at,
    DROP COLUMN banner,
    DROP COLUMN deleted,
    DROP COLUMN inbox_url,
    DROP COLUMN matrix_user_id,
    DROP COLUMN bot_account,
    DROP COLUMN instance_id,
    DROP COLUMN post_count,
    DROP COLUMN post_score,
    DROP COLUMN comment_count,
    DROP COLUMN comment_score;

ALTER TABLE person RENAME COLUMN published_new TO published;

ALTER TABLE person RENAME COLUMN updated_new TO updated;

ALTER TABLE person RENAME COLUMN ap_id_new TO ap_id;

ALTER TABLE person RENAME COLUMN bio_new TO bio;

ALTER TABLE person RENAME COLUMN local_new TO local;

ALTER TABLE person RENAME COLUMN private_key_new TO private_key;

ALTER TABLE person RENAME COLUMN public_key_new TO public_key;

ALTER TABLE person RENAME COLUMN last_refreshed_at_new TO last_refreshed_at;

ALTER TABLE person RENAME COLUMN banner_new TO banner;

ALTER TABLE person RENAME COLUMN deleted_new TO deleted;

ALTER TABLE person RENAME COLUMN inbox_url_new TO inbox_url;

ALTER TABLE person RENAME COLUMN matrix_user_id_new TO matrix_user_id;

ALTER TABLE person RENAME COLUMN bot_account_new TO bot_account;

ALTER TABLE person RENAME COLUMN instance_id_new TO instance_id;

ALTER TABLE person RENAME COLUMN post_count_new TO post_count;

ALTER TABLE person RENAME COLUMN post_score_new TO post_score;

ALTER TABLE person RENAME COLUMN comment_count_new TO comment_count;

ALTER TABLE person RENAME COLUMN comment_score_new TO comment_score;

ALTER TABLE person
    ALTER public_key SET NOT NULL,
    ALTER instance_id SET NOT NULL,
    ADD CONSTRAINT idx_person_actor_id UNIQUE (ap_id);

CREATE INDEX idx_person_local_instance ON person USING btree (local DESC, instance_id);

CREATE UNIQUE INDEX idx_person_lower_actor_id ON person USING btree (lower((ap_id)::text));

CREATE INDEX idx_person_published ON person USING btree (published DESC);

ALTER TABLE ONLY person
    ADD CONSTRAINT person_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE;

-- write existing bans into person table
UPDATE
    person
SET
    (banned,
        ban_expires) = (TRUE,
        subquery.expires)
FROM (
    SELECT
        instance_actions.ban_expires AS expires
    FROM
        instance_actions
        INNER JOIN instance ON instance_actions.instance_id = instance.id
        INNER JOIN person ON person.instance_id = instance.id
    WHERE
        instance_actions.received_ban != NULL) AS subquery;

ALTER TABLE instance_actions
    DROP COLUMN received_ban,
    DROP COLUMN ban_expires;

