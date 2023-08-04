CREATE TABLE site_language (
    id serial PRIMARY KEY,
    site_id int REFERENCES site ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    language_id int REFERENCES
    LANGUAGE ON
    UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    UNIQUE (site_id, language_id)
);

CREATE TABLE community_language (
    id serial PRIMARY KEY,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    language_id int REFERENCES
    LANGUAGE ON
    UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    UNIQUE (community_id, language_id)
);

-- update existing users, sites and communities to have all languages enabled
DO $$
DECLARE
    xid integer;
BEGIN
    FOR xid IN
    SELECT
        id
    FROM
        local_user LOOP
            INSERT INTO local_user_language (local_user_id, language_id) (
                SELECT
                    xid,
                    language.id AS lid
                FROM
                    LANGUAGE);
        END LOOP;
    FOR xid IN
    SELECT
        id
    FROM
        site LOOP
            INSERT INTO site_language (site_id, language_id) (
                SELECT
                    xid,
                    language.id AS lid
                FROM
                    LANGUAGE);
        END LOOP;
    FOR xid IN
    SELECT
        id
    FROM
        community LOOP
            INSERT INTO community_language (community_id, language_id) (
                SELECT
                    xid,
                    language.id AS lid
                FROM
                    LANGUAGE);
        END LOOP;
END;
$$;

