ALTER TABLE post
    DROP COLUMN language_id;

DROP TABLE local_user_language;

DROP TABLE LANGUAGE;

ALTER TABLE local_user RENAME COLUMN interface_language TO lang;

