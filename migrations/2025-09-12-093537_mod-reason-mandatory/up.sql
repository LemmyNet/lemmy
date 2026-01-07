-- provide default value for null rows
UPDATE
    admin_allow_instance
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_ban
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_block_instance
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_purge_comment
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_purge_community
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_purge_person
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_purge_post
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    admin_remove_community
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    mod_ban_from_community
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    mod_lock_comment
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    mod_lock_post
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    mod_remove_comment
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

UPDATE
    mod_remove_post
SET
    reason = 'No reason given'
WHERE
    reason IS NULL;

-- set not null
ALTER TABLE admin_allow_instance
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_ban
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_block_instance
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_purge_comment
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_purge_community
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_purge_person
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_purge_post
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE admin_remove_community
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE mod_ban_from_community
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE mod_lock_comment
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE mod_lock_post
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE mod_remove_comment
    ALTER COLUMN reason SET NOT NULL;

ALTER TABLE mod_remove_post
    ALTER COLUMN reason SET NOT NULL;

