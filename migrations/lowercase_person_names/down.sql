-- This file should undo anything in `up.sql`


-- Remove column constraints

ALTER TABLE person DROP CONSTRAINT person_inbox_url_lowercase_ck;
ALTER TABLE person DROP CONSTRAINT person_actor_id_lowercase_ck;
ALTER TABLE person DROP CONSTRAINT person_actor_id_lowercase_ck;
