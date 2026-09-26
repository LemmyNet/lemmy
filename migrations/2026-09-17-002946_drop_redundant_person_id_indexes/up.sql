-- These need to be `IF EXISTS` because the indexes only exist on servers that ran the old version of the smoosh-tables-together migration.
DROP INDEX IF EXISTS idx_person_actions_person, idx_post_actions_person, idx_comment_actions_person, idx_community_actions_person, idx_instance_actions_person;

