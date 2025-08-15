ALTER TABLE mod_change_community_visibility
    ADD COLUMN reason text,
    ADD COLUMN visibility_new community_visibility;

UPDATE
    mod_change_community_visibility
SET
    visibility_new = visibility;

ALTER TABLE mod_change_community_visibility
    DROP COLUMN visibility;

ALTER TABLE mod_change_community_visibility RENAME COLUMN visibility_new TO visibility;

ALTER TABLE mod_change_community_visibility
    ALTER COLUMN visibility SET NOT NULL;

