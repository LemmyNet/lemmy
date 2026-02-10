-- add ModWarn to constraint checks
ALTER TABLE modlog
    DROP CONSTRAINT IF EXISTS modlog_check;

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
        OR (kind = 'ModWarnComment'
        AND num_nonnulls (target_comment_id, target_person_id) = 2
        AND num_nonnulls (target_community_id, target_instance_id, target_post_id) = 0)
        OR (kind = 'ModLockPost'
        AND num_nonnulls (target_post_id, target_person_id, target_community_id) = 3
        AND num_nonnulls (target_instance_id, target_comment_id) = 0)
        OR (kind = 'ModWarnPost'
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

