ALTER TABLE admin_allow_instance
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_ban
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_block_instance
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_purge_comment
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_purge_community
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_purge_person
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_purge_post
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE admin_remove_community
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE mod_ban_from_community
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE mod_lock_comment
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE mod_lock_post
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE mod_remove_comment
    ALTER COLUMN reason DROP NOT NULL;

ALTER TABLE mod_remove_post
    ALTER COLUMN reason DROP NOT NULL;

