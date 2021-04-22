ALTER TABLE comment DROP COLUMN language;
ALTER TABLE post DROP COLUMN language;
ALTER TABLE private_message DROP COLUMN language;

DROP TABLE discussion_languages;
ALTER TABLE local_user RENAME COLUMN interface_language TO lang;
