ALTER TABLE modlog
    DROP CONSTRAINT modlog_mod_fkey,
    DROP CONSTRAINT modlog_target_person_fkey,
    DROP CONSTRAINT modlog_target_community_fkey,
    DROP CONSTRAINT modlog_target_post_fkey,
    DROP CONSTRAINT modlog_target_comment_fkey,
    DROP CONSTRAINT modlog_target_instance_fkey,
    DROP CONSTRAINT modlog_bulk_action_parent_fkey;

DROP INDEX idx_modlog_mod, idx_modlog_kind, idx_modlog_target_person, idx_modlog_target_community, idx_modlog_target_post, idx_modlog_target_comment, idx_modlog_target_instance, idx_modlog_published_id
