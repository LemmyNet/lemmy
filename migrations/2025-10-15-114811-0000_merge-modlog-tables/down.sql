CREATE TABLE mod_add_to_community (
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    other_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    removed boolean DEFAULT FALSE NOT NULL
);

ALTER SEQUENCE mod_add_to_community_id_seq
    RENAME TO mod_add_community_id_seq;

ALTER TABLE mod_add_to_community RENAME CONSTRAINT mod_add_to_community_community_id_fkey TO mod_add_community_community_id_fkey;

ALTER TABLE mod_add_to_community RENAME CONSTRAINT mod_add_to_community_mod_person_id_fkey TO mod_add_community_mod_person_id_fkey;

ALTER TABLE mod_add_to_community RENAME CONSTRAINT mod_add_to_community_other_person_id_fkey TO mod_add_community_other_person_id_fkey;

ALTER TABLE mod_add_to_community RENAME CONSTRAINT mod_add_to_community_pkey TO mod_add_community_pkey;

CREATE TABLE admin_purge_comment (
    admin_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    post_id integer NOT NULL REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE admin_add (
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    other_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    removed boolean DEFAULT FALSE NOT NULL
);

ALTER SEQUENCE admin_add_id_seq
    RENAME TO mod_add_id_seq;

ALTER TABLE admin_add RENAME CONSTRAINT admin_add_mod_person_id_fkey TO mod_add_mod_person_id_fkey;

ALTER TABLE admin_add RENAME CONSTRAINT admin_add_other_person_id_fkey TO mod_add_other_person_id_fkey;

ALTER TABLE admin_add RENAME CONSTRAINT admin_add_pkey TO mod_add_pkey;

CREATE TABLE mod_transfer_community (
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    other_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE admin_allow_instance (
    admin_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    allowed boolean NOT NULL,
    id serial PRIMARY KEY,
    instance_id integer NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE mod_lock_post (
    id serial PRIMARY KEY,
    locked boolean DEFAULT TRUE NOT NULL,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    post_id integer NOT NULL REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE mod_remove_post (
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    post_id integer NOT NULL REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL,
    removed boolean DEFAULT TRUE NOT NULL
);

CREATE TABLE mod_change_community_visibility (
    community_id integer NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    visibility community_visibility NOT NULL
);

CREATE TABLE mod_remove_comment (
    comment_id integer NOT NULL REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL,
    removed boolean DEFAULT TRUE NOT NULL
);

CREATE TABLE admin_remove_community (
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL,
    removed boolean DEFAULT TRUE NOT NULL
);

ALTER SEQUENCE admin_remove_community_id_seq
    RENAME TO mod_remove_community_id_seq;

ALTER TABLE admin_remove_community RENAME CONSTRAINT admin_remove_community_community_id_fkey TO mod_remove_community_community_id_fkey;

ALTER TABLE admin_remove_community RENAME CONSTRAINT admin_remove_community_mod_person_id_fkey TO mod_remove_community_mod_person_id_fkey;

ALTER TABLE admin_remove_community RENAME CONSTRAINT admin_remove_community_pkey TO mod_remove_community_pkey;

CREATE TABLE mod_lock_comment (
    comment_id integer NOT NULL REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    locked boolean DEFAULT TRUE NOT NULL,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE mod_feature_post (
    featured boolean DEFAULT TRUE NOT NULL,
    id serial PRIMARY KEY,
    is_featured_community boolean DEFAULT TRUE NOT NULL,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    post_id integer NOT NULL REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

ALTER SEQUENCE mod_feature_post_id_seq
    RENAME TO mod_sticky_post_id_seq;

ALTER TABLE mod_feature_post RENAME CONSTRAINT mod_feature_post_mod_person_id_fkey TO mod_sticky_post_mod_person_id_fkey;

ALTER TABLE mod_feature_post RENAME CONSTRAINT mod_feature_post_pkey TO mod_sticky_post_pkey;

ALTER TABLE mod_feature_post RENAME CONSTRAINT mod_feature_post_post_id_fkey TO mod_sticky_post_post_id_fkey;

CREATE TABLE admin_block_instance (
    admin_person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    blocked boolean NOT NULL,
    expires_at timestamp with time zone,
    id serial PRIMARY KEY,
    instance_id integer NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE admin_ban (
    banned boolean DEFAULT TRUE NOT NULL,
    expires_at timestamp with time zone,
    id serial PRIMARY KEY,
    instance_id integer NOT NULL REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE,
    mod_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    other_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

ALTER SEQUENCE admin_ban_id_seq
    RENAME TO mod_ban_id_seq;

ALTER TABLE admin_ban RENAME CONSTRAINT admin_ban_instance_id_fkey TO mod_ban_instance_id_fkey;

ALTER TABLE admin_ban RENAME CONSTRAINT admin_ban_mod_person_id_fkey TO mod_ban_mod_person_id_fkey;

ALTER TABLE admin_ban RENAME CONSTRAINT admin_ban_other_person_id_fkey TO mod_ban_other_person_id_fkey;

ALTER TABLE admin_ban RENAME CONSTRAINT admin_ban_pkey TO mod_ban_pkey;

CREATE TABLE admin_purge_post (
    admin_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE admin_purge_person (
    admin_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE admin_purge_community (
    admin_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    id serial PRIMARY KEY,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL
);

CREATE TABLE mod_ban_from_community (
    id serial PRIMARY KEY,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text NOT NULL,
    mod_person_id int NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    community_id int NOT NULL REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    expires_at timestamp with time zone,
    other_person_id integer NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    banned bool NOT NULL DEFAULT TRUE
);

CREATE TABLE modlog_combined (
    id serial PRIMARY KEY,
    published_at timestamptz NOT NULL,
    admin_allow_instance_id int UNIQUE REFERENCES admin_allow_instance ON UPDATE CASCADE ON DELETE CASCADE,
    admin_block_instance_id int UNIQUE REFERENCES admin_block_instance ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_comment_id int UNIQUE REFERENCES admin_purge_comment ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_community_id int UNIQUE REFERENCES admin_purge_community ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_person_id int UNIQUE REFERENCES admin_purge_person ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_post_id int UNIQUE REFERENCES admin_purge_post ON UPDATE CASCADE ON DELETE CASCADE,
    admin_add_id int UNIQUE REFERENCES admin_add ON UPDATE CASCADE ON DELETE CASCADE,
    mod_add_to_community_id int UNIQUE REFERENCES mod_add_to_community ON UPDATE CASCADE ON DELETE CASCADE,
    admin_ban_id int UNIQUE REFERENCES admin_ban ON UPDATE CASCADE ON DELETE CASCADE,
    mod_ban_from_community_id int UNIQUE REFERENCES mod_ban_from_community ON UPDATE CASCADE ON DELETE CASCADE,
    mod_feature_post_id int UNIQUE REFERENCES mod_feature_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_change_community_visibility_id int REFERENCES mod_change_community_visibility ON UPDATE CASCADE ON DELETE CASCADE,
    mod_lock_post_id int UNIQUE REFERENCES mod_lock_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_lock_comment_id int UNIQUE REFERENCES mod_lock_comment ON UPDATE CASCADE ON DELETE CASCADE,
    mod_remove_comment_id int UNIQUE REFERENCES mod_remove_comment ON UPDATE CASCADE ON DELETE CASCADE,
    admin_remove_community_id int UNIQUE REFERENCES admin_remove_community ON UPDATE CASCADE ON DELETE CASCADE,
    mod_remove_post_id int UNIQUE REFERENCES mod_remove_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_transfer_community_id int UNIQUE REFERENCES mod_transfer_community ON UPDATE CASCADE ON DELETE CASCADE
);

ALTER TABLE modlog_combined
    ADD CONSTRAINT modlog_combined_check CHECK (num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, admin_add_id, mod_add_to_community_id, admin_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_change_community_visibility_id, mod_lock_post_id, mod_remove_comment_id, admin_remove_community_id, mod_remove_post_id, mod_transfer_community_id, mod_lock_comment_id) = 1);

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_admin_add_id_fkey TO modlog_combined_mod_add_id_fkey;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_admin_add_id_key TO modlog_combined_mod_add_id_key;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_admin_ban_id_fkey TO modlog_combined_mod_ban_id_fkey;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_admin_ban_id_key TO modlog_combined_mod_ban_id_key;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_admin_remove_community_id_fkey TO modlog_combined_mod_remove_community_id_fkey;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_admin_remove_community_id_key TO modlog_combined_mod_remove_community_id_key;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_mod_add_to_community_id_key TO modlog_combined_mod_add_community_id_key;

ALTER TABLE modlog_combined RENAME CONSTRAINT modlog_combined_mod_add_to_community_id_fkey TO modlog_combined_mod_add_community_id_fkey;

ALTER TABLE notification
    ADD COLUMN admin_add_id int REFERENCES admin_add ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_add_to_community_id int REFERENCES mod_add_to_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN admin_ban_id int REFERENCES admin_ban ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_ban_from_community_id int REFERENCES mod_ban_from_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_post_id int REFERENCES mod_lock_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_remove_comment_id int REFERENCES mod_remove_comment ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN admin_remove_community_id int REFERENCES admin_remove_community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_remove_post_id int REFERENCES mod_remove_post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_comment_id int REFERENCES mod_lock_comment ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_transfer_community_id int REFERENCES mod_transfer_community ON UPDATE CASCADE ON DELETE CASCADE,
    DROP COLUMN modlog_id;

ALTER TABLE notification
    DROP CONSTRAINT IF EXISTS notification_check;

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id, admin_add_id, mod_add_to_community_id, admin_ban_id, mod_ban_from_community_id, mod_lock_post_id, mod_remove_post_id, mod_lock_comment_id, mod_remove_comment_id, admin_remove_community_id, mod_transfer_community_id) = 1);

DROP TABLE modlog;

DROP TYPE modlog_kind;

