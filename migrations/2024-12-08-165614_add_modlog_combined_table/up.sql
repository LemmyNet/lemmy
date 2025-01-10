-- First, rename all the when_ columns on the modlog to published
ALTER TABLE admin_allow_instance RENAME COLUMN when_ TO published;

ALTER TABLE admin_block_instance RENAME COLUMN when_ TO published;

ALTER TABLE admin_purge_comment RENAME COLUMN when_ TO published;

ALTER TABLE admin_purge_community RENAME COLUMN when_ TO published;

ALTER TABLE admin_purge_person RENAME COLUMN when_ TO published;

ALTER TABLE admin_purge_post RENAME COLUMN when_ TO published;

ALTER TABLE mod_add RENAME COLUMN when_ TO published;

ALTER TABLE mod_add_community RENAME COLUMN when_ TO published;

ALTER TABLE mod_ban RENAME COLUMN when_ TO published;

ALTER TABLE mod_ban_from_community RENAME COLUMN when_ TO published;

ALTER TABLE mod_feature_post RENAME COLUMN when_ TO published;

ALTER TABLE mod_hide_community RENAME COLUMN when_ TO published;

ALTER TABLE mod_lock_post RENAME COLUMN when_ TO published;

ALTER TABLE mod_remove_comment RENAME COLUMN when_ TO published;

ALTER TABLE mod_remove_community RENAME COLUMN when_ TO published;

ALTER TABLE mod_remove_post RENAME COLUMN when_ TO published;

ALTER TABLE mod_transfer_community RENAME COLUMN when_ TO published;

-- Creates combined tables for
-- modlog: (17 tables)
-- admin_allow_instance
-- admin_block_instance
-- admin_purge_comment
-- admin_purge_community
-- admin_purge_person
-- admin_purge_post
-- mod_add
-- mod_add_community
-- mod_ban
-- mod_ban_from_community
-- mod_feature_post
-- mod_hide_community
-- mod_lock_post
-- mod_remove_comment
-- mod_remove_community
-- mod_remove_post
-- mod_transfer_community
CREATE TABLE modlog_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    admin_allow_instance_id int UNIQUE REFERENCES admin_allow_instance ON UPDATE CASCADE ON DELETE CASCADE,
    admin_block_instance_id int UNIQUE REFERENCES admin_block_instance ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_comment_id int UNIQUE REFERENCES admin_purge_comment ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_community_id int UNIQUE REFERENCES admin_purge_community ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_person_id int UNIQUE REFERENCES admin_purge_person ON UPDATE CASCADE ON DELETE CASCADE,
    admin_purge_post_id int UNIQUE REFERENCES admin_purge_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_add_id int UNIQUE REFERENCES mod_add ON UPDATE CASCADE ON DELETE CASCADE,
    mod_add_community_id int UNIQUE REFERENCES mod_add_community ON UPDATE CASCADE ON DELETE CASCADE,
    mod_ban_id int UNIQUE REFERENCES mod_ban ON UPDATE CASCADE ON DELETE CASCADE,
    mod_ban_from_community_id int UNIQUE REFERENCES mod_ban_from_community ON UPDATE CASCADE ON DELETE CASCADE,
    mod_feature_post_id int UNIQUE REFERENCES mod_feature_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_hide_community_id int UNIQUE REFERENCES mod_hide_community ON UPDATE CASCADE ON DELETE CASCADE,
    mod_lock_post_id int UNIQUE REFERENCES mod_lock_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_remove_comment_id int UNIQUE REFERENCES mod_remove_comment ON UPDATE CASCADE ON DELETE CASCADE,
    mod_remove_community_id int UNIQUE REFERENCES mod_remove_community ON UPDATE CASCADE ON DELETE CASCADE,
    mod_remove_post_id int UNIQUE REFERENCES mod_remove_post ON UPDATE CASCADE ON DELETE CASCADE,
    mod_transfer_community_id int UNIQUE REFERENCES mod_transfer_community ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_community_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_community_id, mod_ban_id, mod_ban_from_community_id, mod_feature_post_id, mod_hide_community_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_community_id, mod_remove_post_id, mod_transfer_community_id) = 1)
);

CREATE INDEX idx_modlog_combined_published ON modlog_combined (published DESC, id DESC);

-- Updating the history
-- Not doing a union all here, because there's way too many null columns
INSERT INTO modlog_combined (published, admin_allow_instance_id)
SELECT
    published,
    id
FROM
    admin_allow_instance;

INSERT INTO modlog_combined (published, admin_block_instance_id)
SELECT
    published,
    id
FROM
    admin_block_instance;

INSERT INTO modlog_combined (published, admin_purge_comment_id)
SELECT
    published,
    id
FROM
    admin_purge_comment;

INSERT INTO modlog_combined (published, admin_purge_community_id)
SELECT
    published,
    id
FROM
    admin_purge_community;

INSERT INTO modlog_combined (published, admin_purge_person_id)
SELECT
    published,
    id
FROM
    admin_purge_person;

INSERT INTO modlog_combined (published, admin_purge_post_id)
SELECT
    published,
    id
FROM
    admin_purge_post;

INSERT INTO modlog_combined (published, mod_add_id)
SELECT
    published,
    id
FROM
    mod_add;

INSERT INTO modlog_combined (published, mod_add_community_id)
SELECT
    published,
    id
FROM
    mod_add_community;

INSERT INTO modlog_combined (published, mod_ban_id)
SELECT
    published,
    id
FROM
    mod_ban;

INSERT INTO modlog_combined (published, mod_ban_from_community_id)
SELECT
    published,
    id
FROM
    mod_ban_from_community;

INSERT INTO modlog_combined (published, mod_feature_post_id)
SELECT
    published,
    id
FROM
    mod_feature_post;

INSERT INTO modlog_combined (published, mod_hide_community_id)
SELECT
    published,
    id
FROM
    mod_hide_community;

INSERT INTO modlog_combined (published, mod_lock_post_id)
SELECT
    published,
    id
FROM
    mod_lock_post;

INSERT INTO modlog_combined (published, mod_remove_comment_id)
SELECT
    published,
    id
FROM
    mod_remove_comment;

INSERT INTO modlog_combined (published, mod_remove_community_id)
SELECT
    published,
    id
FROM
    mod_remove_community;

INSERT INTO modlog_combined (published, mod_remove_post_id)
SELECT
    published,
    id
FROM
    mod_remove_post;

INSERT INTO modlog_combined (published, mod_transfer_community_id)
SELECT
    published,
    id
FROM
    mod_transfer_community;

