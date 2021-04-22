ALTER TABLE comment ADD COLUMN language TEXT NOT NULL;
ALTER TABLE post ADD COLUMN language TEXT NOT NULL;
ALTER TABLE private_message ADD COLUMN language TEXT NOT NULL;

CREATE TABLE discussion_languages(id INTEGER PRIMARY KEY, local_user_id INT NOT NULL, language TEXT NOT NULL);
ALTER TABLE local_user RENAME COLUMN lang TO interface_language;
