-- This adds historical data for existing modlog rows
ALTER TABLE modlog
    DROP CONSTRAINT IF EXISTS modlog_check;

-- AdminAdd
UPDATE
    modlog
SET
    target_instance_id = p.instance_id
FROM
    person p
WHERE
    kind = 'AdminAdd'
    AND target_person_id = p.id;

-- AdminFeaturePostSite
UPDATE
    modlog
SET
    target_community_id = p.community_id,
    target_instance_id = co.instance_id
FROM
    post p,
    community co
WHERE
    kind = 'AdminFeaturePostSite'
    AND target_post_id = p.id
    AND p.community_id = co.id;

-- AdminRemoveCommunity
UPDATE
    modlog
SET
    target_instance_id = co.instance_id
FROM
    community co
WHERE
    kind = 'AdminRemoveCommunity'
    AND target_community_id = co.id;

-- target_comment_id
UPDATE
    modlog
SET
    target_community_id = c.community_id,
    target_post_id = c.post_id
FROM
    comment c
WHERE
    target_comment_id = c.id;

-- target_post_id
UPDATE
    modlog
SET
    target_community_id = p.community_id
FROM
    post p
WHERE
    target_post_id = p.id;

ALTER TABLE modlog
    ADD CHECK ((kind = 'AdminAdd'
        AND num_nonnulls (target_person_id, target_instance_id) = 2
        AND num_nonnulls (target_community_id, target_post_id, target_comment_id) = 0)
        OR (kind = 'AdminBan'
        AND num_nonnulls (target_person_id, target_instance_id) = 2
        AND num_nonnulls (target_community_id, target_post_id, target_comment_id) = 0)
        OR (kind = 'ModRemovePost'
        AND num_nonnulls (target_post_id, target_community_id, target_person_id) = 3
        AND num_nonnulls (target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModRemoveComment'
        AND num_nonnulls (target_comment_id, target_person_id, target_post_id, target_community_id) = 4
        AND num_nonnulls (target_instance_id) = 0)
        OR (kind = 'ModLockComment'
        AND num_nonnulls (target_comment_id, target_person_id, target_post_id, target_community_id) = 4
        AND num_nonnulls (target_instance_id) = 0)
        OR (kind = 'ModWarnComment'
        AND num_nonnulls (target_comment_id, target_person_id, target_post_id, target_community_id) = 4
        AND num_nonnulls (target_instance_id) = 0)
        OR (kind = 'ModLockPost'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id) = 3
        AND num_nonnulls (target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModWarnPost'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id) = 3
        AND num_nonnulls (target_instance_id, target_comment_id) = 0)
        OR (kind = 'AdminRemoveCommunity'
        AND num_nonnulls (target_community_id, target_instance_id) = 2
        AND num_nonnulls (target_post_id, target_comment_id) = 0)
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
        AND num_nonnulls (target_post_id, target_community_id, target_instance_id) = 3
        AND num_nonnulls (target_person_id, target_comment_id) = 0));

