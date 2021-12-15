-- Your SQL goes here


-- Set person names, actor_id, and inbox_url to lowercase

UPDATE person SET name=lower(name);
UPDATE person SET actor_id=lower(actor_id);
UPDATE person SET inbox_url=lower(inbox_url);



-- Add a lowecase enforcement check to these columns

ALTER TABLE person
  ADD CONSTRAINT person_name_lowercase_ck
  CHECK (name = lower(name));

ALTER TABLE person
  ADD CONSTRAINT person_actor_id_lowercase_ck
  CHECK (actor_id = lower(actor_id));

ALTER TABLE person
  ADD CONSTRAINT person_inbox_url_lowercase_ck
  CHECK (inbox_url = lower(inbox_url));
