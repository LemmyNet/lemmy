-- New enum with all possible mod actions
-- TODO: We could also remove the Admin/Mod prefix
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
    'ModFeaturePostCommunity',
    'AdminFeaturePostSite',
    'ModChangeCommunityVisibility',
    'ModLockPost',
    'ModRemoveComment',
    'AdminRemoveCommunity',
    'ModRemovePost',
    'ModTransferCommunity',
    'ModLockComment'
);

-- New table with data for all mod actions
CREATE TABLE modlog (
    id serial PRIMARY KEY,
    kind modlog_kind NOT NULL,
    -- Used to be `revert`, but that makes little sense for things like feature post or
    -- transfer community. Instead we use this which means values have to be inverted.
    is_revert boolean NOT NULL,
    -- Not using `references person` for any of the foreign keys to avoid modlog entries
    -- disappearing if the mod or any target gets purged.
    mod_id int NOT NULL,
    -- For some actions reason is quite pointless so leave it optional (eg add admin, feature post)
    reason text,
    target_person_id int,
    target_community_id int,
    target_post_id int,
    target_comment_id int,
    target_instance_id int,
    expires_at timestamptz,
    published_at timestamptz NOT NULL DEFAULT now()
);

-- Most mod actions can have only one target. We could make this much more specific and state
-- which exact column must be set for each kind but that would be excessive.
ALTER TABLE modlog
    ADD CHECK ((kind = 'AdminAdd'
        AND num_nonnulls (target_person_id) = 1
        AND num_nonnulls (target_community_id, target_post_id, target_comment_id, target_instance_id) = 0)
        OR (kind = 'AdminBan'
        AND num_nonnulls (target_person_id, target_instance_id) = 2
        AND num_nonnulls (target_community_id, target_post_id, target_comment_id) = 0)
        OR (kind = 'ModRemovePost'
        AND num_nonnulls (target_post_id, target_person_id) = 2
        AND num_nonnulls (target_community_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModRemoveComment'
        AND num_nonnulls (target_comment_id, target_person_id, target_post_id) = 3
        AND num_nonnulls (target_community_id, target_instance_id) = 0)
        OR (kind = 'ModLockComment'
        AND num_nonnulls (target_comment_id, target_person_id) = 2
        AND num_nonnulls (target_community_id, target_instance_id, target_post_id) = 0)
        OR (kind = 'ModLockPost'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id) = 3
        AND num_nonnulls (target_instance_id, target_comment_id) = 0)
        OR (kind = 'AdminRemoveCommunity'
        AND num_nonnulls (target_community_id) = 1
        -- target_person_id (community owner) can be either null or not null here
        AND num_nonnulls (target_post_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModChangeCommunityVisibility'
        AND num_nonnulls (target_community_id) = 1
        AND num_nonnulls (target_post_id, target_instance_id, target_person_id, target_comment_id) = 0)
        OR (kind = 'ModBanFromCommunity'
        AND num_nonnulls (target_community_id, target_person_id) = 2
        AND num_nonnulls (target_post_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModAddToCommunity'
        AND num_nonnulls (target_community_id, target_person_id) = 2
        AND num_nonnulls (target_post_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModTransferCommunity'
        AND num_nonnulls (target_community_id, target_person_id) = 2
        AND num_nonnulls (target_post_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'AdminAllowInstance'
        AND num_nonnulls (target_instance_id) = 1
        AND num_nonnulls (target_post_id, target_person_id, target_community_id, target_comment_id) = 0)
        OR (kind = 'AdminBlockInstance'
        AND num_nonnulls (target_instance_id) = 1
        AND num_nonnulls (target_post_id, target_person_id, target_community_id, target_comment_id) = 0)
        OR (kind = 'AdminPurgeComment'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id) = 3
        AND num_nonnulls (target_instance_id, target_comment_id) = 0)
        OR (kind = 'AdminPurgePost'
        AND num_nonnulls (target_community_id) = 1
        AND num_nonnulls (target_post_id, target_person_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'AdminPurgeCommunity'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'AdminPurgePerson'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id, target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModFeaturePostCommunity'
        AND num_nonnulls (target_post_id, target_community_id) = 2
        AND num_nonnulls (target_instance_id, target_person_id, target_comment_id) = 0)
        OR (kind = 'AdminFeaturePostSite'
        AND num_nonnulls (target_post_id) = 1
        AND num_nonnulls (target_instance_id, target_person_id, target_comment_id, target_community_id) = 0));

-- copy old data to new table
INSERT INTO modlog (kind, is_revert, mod_id, target_person_id, published_at)
SELECT
    'AdminAdd',
    NOT removed,
    mod_person_id,
    other_person_id,
    published_at
FROM
    admin_add;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_person_id, target_instance_id, published_at)
SELECT
    'AdminBan',
    reason,
    NOT banned,
    mod_person_id,
    other_person_id,
    p.instance_id,
    a. published_at
FROM
    admin_ban a
    INNER JOIN person p ON p.id = mod_person_id;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_post_id, target_person_id, published_at)
SELECT
    'ModRemovePost',
    reason,
    NOT m.removed,
    mod_person_id,
    post_id,
    p.creator_id,
    m.published_at
FROM
    mod_remove_post m
    INNER JOIN post p ON p.id = post_id;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_comment_id, target_person_id, target_post_id, published_at)
SELECT
    'ModRemoveComment',
    reason,
    NOT m.removed,
    mod_person_id,
    comment_id,
    c.creator_id,
    c.post_id,
    m.published_at
FROM
    mod_remove_comment m
    INNER JOIN comment c ON c.id = comment_id;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_comment_id, target_person_id, published_at)
SELECT
    'ModLockComment',
    reason,
    NOT m.LOCKED,
    mod_person_id,
    comment_id,
    c.creator_id,
    m.published_at
FROM
    mod_lock_comment m
    INNER JOIN comment c ON c.id = comment_id;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_post_id, target_person_id, target_community_id, published_at)
SELECT
    'ModLockPost',
    reason,
    NOT m.LOCKED,
    mod_person_id,
    post_id,
    p.creator_id,
    p.community_id,
    m.published_at
FROM
    mod_lock_post m
    INNER JOIN post p ON p.id = post_id;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_community_id, published_at)
SELECT
    'AdminRemoveCommunity',
    reason,
    NOT removed,
    mod_person_id,
    community_id,
    published_at
FROM
    admin_remove_community;

INSERT INTO modlog (kind, is_revert, mod_id, target_community_id, published_at)
SELECT
    'ModChangeCommunityVisibility',
    FALSE,
    mod_person_id,
    community_id,
    published_at
FROM
    mod_change_community_visibility;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_community_id, target_person_id, expires_at, published_at)
SELECT
    'ModBanFromCommunity',
    reason,
    NOT banned,
    mod_person_id,
    community_id,
    other_person_id,
    expires_at,
    published_at
FROM
    mod_ban_from_community;

INSERT INTO modlog (kind, is_revert, mod_id, target_community_id, target_person_id, published_at)
SELECT
    'ModAddToCommunity',
    NOT removed,
    mod_person_id,
    community_id,
    other_person_id,
    published_at
FROM
    mod_add_to_community;

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

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_instance_id, published_at)
SELECT
    'AdminAllowInstance',
    reason,
    NOT allowed,
    admin_person_id,
    instance_id,
    published_at
FROM
    admin_allow_instance;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_instance_id, published_at)
SELECT
    'AdminBlockInstance',
    reason,
    NOT blocked,
    admin_person_id,
    instance_id,
    published_at
FROM
    admin_block_instance;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_post_id, target_person_id, target_community_id, published_at)
SELECT
    'AdminPurgeComment',
    reason,
    FALSE,
    admin_person_id,
    post_id,
    p.creator_id,
    p.community_id,
    a.published_at
FROM
    admin_purge_comment a
    INNER JOIN post p ON p.id = post_id;

INSERT INTO modlog (kind, reason, is_revert, mod_id, target_community_id, published_at)
SELECT
    'AdminPurgePost',
    reason,
    FALSE,
    admin_person_id,
    community_id,
    published_at
FROM
    admin_purge_post;

INSERT INTO modlog (kind, reason, is_revert, mod_id, published_at)
SELECT
    'AdminPurgeCommunity',
    reason,
    FALSE,
    admin_person_id,
    published_at
FROM
    admin_purge_community;

INSERT INTO modlog (kind, reason, is_revert, mod_id, published_at)
SELECT
    'AdminPurgePerson',
    reason,
    FALSE,
    admin_person_id,
    published_at
FROM
    admin_purge_person;

INSERT INTO modlog (kind, is_revert, mod_id, target_post_id, target_community_id, published_at)
SELECT
    'ModFeaturePostCommunity',
    NOT featured,
    mod_person_id,
    post_id,
    post.community_id,
    m.published_at
FROM
    mod_feature_post m
    INNER JOIN post ON post.id = m.post_id
WHERE
    is_featured_community;

INSERT INTO modlog (kind, is_revert, mod_id, target_post_id, published_at)
SELECT
    'AdminFeaturePostSite',
    NOT featured,
    mod_person_id,
    post_id,
    published_at
FROM
    mod_feature_post
WHERE
    NOT is_featured_community;

ALTER TABLE notification
    DROP CONSTRAINT IF EXISTS notification_check;

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

DELETE FROM notification
WHERE post_id IS NULL
    AND comment_id IS NULL
    AND private_message_id IS NULL
    AND modlog_id IS NULL;

ALTER TABLE notification
    ADD CONSTRAINT notification_check CHECK (num_nonnulls (post_id, comment_id, private_message_id, modlog_id) = 1);

CREATE INDEX idx_notification_modlog_id ON notification USING btree (modlog_id)
WHERE (modlog_id IS NOT NULL);

DROP TABLE modlog_combined, admin_add, admin_allow_instance, admin_ban, admin_block_instance, admin_remove_community, admin_purge_comment, admin_purge_community, admin_purge_person, admin_purge_post, mod_add_to_community, mod_ban_from_community, mod_change_community_visibility, mod_feature_post, mod_lock_comment, mod_lock_post, mod_remove_comment, mod_remove_post, mod_transfer_community;

