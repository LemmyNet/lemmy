-- new enum with all possible mod actions
CREATE TYPE modlog_kind AS enum (
    'AdminAdd',
    'AdminBan',
    'AdminAllowInstance',
    'AdminBlockInstance',
    'AdminPurgeComment',
    'AdminPurgeCommunity',
    'AdminPurgePerson',
    'AdminPurgePost',
    'ModAddToCommunity',
    'ModBanFromCommunity',
    'ModFeaturePost',
    'ModChangeCommunityVisibility',
    'ModLockPost',
    'ModRemoveComment',
    'AdminRemoveCommunity',
    'ModRemovePost',
    'ModTransferCommunity',
    'ModLockComment'
);

-- new table with data for all mod actions
CREATE TABLE modlog (
    id serial PRIMARY KEY,
    kind modlog_kind NOT NULL,
    -- For some actions reason is quite pointless so leave it optional (eg add admin, feature post)
    reason text,
    is_revert boolean NOT NULL,
    mod_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    target_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    target_community_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    target_post_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    target_comment_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    target_instance_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    reverted_modlog_id int REFERENCES modlog ON UPDATE CASCADE ON DELETE CASCADE,
    expires_at timestamptz,
    published_at timestamptz NOT NULL DEFAULT now()
);

-- Most mod actions can have only one target. We could make this much more specific and state
-- which exact column must be set for each kind but that would be excessive.
ALTER TABLE modlog
    ADD CHECK (num_nonnulls (target_person_id, target_community_id, target_post_id, target_comment_id, target_instance_id) = 1
        OR ((kind = 'ModBanFromCommunity'
        OR kind = 'ModTransferCommunity')
        AND num_nonnulls (target_person_id, target_community_id, target_post_id, target_comment_id, target_instance_id) = 2));

-- copy old data to new table
INSERT INTO modlog (kind, is_revert, mod_id, target_person_id, published_at)
SELECT
    'AdminAdd',
    removed,
    mod_person_id,
    other_person_id,
    published_at
FROM
    admin_add;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_person_id, published_at)
SELECT
    'AdminBan',
    reason,
    banned,
    mod_person_id,
    other_person_id,
    published_at
FROM
    admin_ban;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_instance_id, published_at)
SELECT
    'AdminAllowInstance',
    reason,
    allowed,
    admin_person_id,
    instance_id,
    published_at
FROM
    admin_allow_instance;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_instance_id, published_at)
SELECT
    'AdminBlockInstance',
    reason,
    blocked,
    admin_person_id,
    instance_id,
    published_at
FROM
    admin_block_instance;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_comment_id, published_at)
SELECT
    'AdminPurgeComment',
    reason,
    FALSE,
    admin_person_id,
    -- old table has wrong column name
    post_id,
    published_at
FROM
    admin_purge_comment;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_post_id, published_at)
SELECT
    'AdminPurgePost',
    reason,
    FALSE,
    admin_person_id,
    post_id,
    published_at
FROM
    admin_purge_comment;

-- This one is not possible because admin_purge_community is missing community_id
-- INSERT INTO modlog (kind, reason, is_revert, mod_id, target_community_id, published_at)
-- SELECT 'AdminPurgeCommunity', reason, FALSE, admin_person_id, community_id,published_at
-- FROM admin_purge_community;
-- Same for admin_purge_person
-- INSERT INTO modlog (kind, reason, is_revert, mod_id, target_person_id, published_at)
-- SELECT 'AdminPurgePerson',reason, FALSE,admin_person_id, person_id, published_at
-- FROM admin_purge_person;
INSERT INTO modlog (kind, is_revert, mod_id, target_person_id, published_at)
SELECT
    'ModAddToCommunity',
    removed,
    mod_person_id,
    other_person_id,
    published_at
FROM
    mod_add_to_community;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_community_id, target_person_id, expires_at, published_at)
SELECT
    'ModBanFromCommunity',
    reason,
    banned,
    mod_person_id,
    community_id,
    other_person_id,
    expires_at,
    published_at
FROM
    mod_ban_from_community;

INSERT INTO modlog (kind, is_revert, mod_id, target_post_id, published_at)
SELECT
    'ModFeaturePost',
    featured,
    mod_person_id,
    post_id,
    published_at
FROM
    mod_feature_post;

INSERT INTO modlog (kind, is_revert, mod_id, target_community_id, published_at)
SELECT
    'ModChangeCommunityVisibility',
    FALSE,
    mod_person_id,
    community_id,
    published_at
FROM
    mod_change_community_visibility;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_post_id, published_at)
SELECT
    'ModLockPost',
    reason,
    LOCKED,
    mod_person_id,
    post_id,
    published_at
FROM
    mod_lock_post;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_comment_id, published_at)
SELECT
    'ModLockComment',
    reason,
    LOCKED,
    mod_person_id,
    comment_id,
    published_at
FROM
    mod_lock_comment;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_comment_id, published_at)
SELECT
    'ModRemoveComment',
    reason,
    removed,
    mod_person_id,
    comment_id,
    published_at
FROM
    mod_remove_comment;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_community_id, published_at)
SELECT
    'AdminRemoveCommunity',
    reason,
    removed,
    mod_person_id,
    community_id,
    published_at
FROM
    admin_remove_community;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_post_id, published_at)
SELECT
    'ModRemovePost',
    reason,
    removed,
    mod_person_id,
    post_id,
    published_at
FROM
    mod_remove_post;

INSERT INTO modlog (kind, is_revert, mod_id, target_community_id, target_person_id, published_at)
SELECT
    'ModTransferCommunity',
    FALSE,
    mod_person_id,
    community_id,
    other_person_id,
    published_at
FROM
    mod_transfer_community;

-- Rewrite notifications to reference new modlog table. This is not used in production yet
-- so no need to copy over data.
ALTER TABLE notification
    DROP COLUMN admin_add_id,
    DROP COLUMN mod_add_to_community_id,
    DROP COLUMN admin_ban_id,
    DROP COLUMN mod_ban_from_community_id,
    DROP COLUMN mod_lock_post_id,
    DROP COLUMN mod_remove_comment_id,
    DROP COLUMN admin_remove_community_id,
    DROP COLUMN mod_remove_post_id,
    DROP COLUMN mod_lock_comment_id,
    DROP COLUMN mod_transfer_community_id,
    ADD COLUMN modlog_id int REFERENCES modlog ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE notification
    DROP CONSTRAINT IF EXISTS notification_check;

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id, modlog_id) = 1);

DROP TABLE modlog_combined, admin_add, admin_allow_instance, admin_ban, admin_block_instance, admin_remove_community, admin_purge_comment, admin_purge_community, admin_purge_person, admin_purge_post, mod_add_to_community, mod_ban_from_community, mod_change_community_visibility, mod_feature_post, mod_lock_comment, mod_lock_post, mod_remove_comment, mod_remove_post, mod_transfer_community;

