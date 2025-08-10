-- Adding a lock reason field to mod_lock_post
ALTER TABLE mod_lock_post
    ADD COLUMN reason text;

