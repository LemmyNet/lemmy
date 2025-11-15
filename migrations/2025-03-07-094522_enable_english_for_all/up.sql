-- enable english for all users on instances with all languages enabled.
-- Fix for https://github.com/LemmyNet/lemmy/pull/5485
DO $$
BEGIN
    IF (
        SELECT
            count(*)
        FROM
            site_language
                INNER JOIN local_site ON site_language.site_id = local_site.site_id) = 184 THEN
        INSERT INTO local_user_language (local_user_id, language_id)
        SELECT
            local_user_id,
            37
        FROM
            local_user_language
        GROUP BY
            local_user_id
        HAVING
            NOT (37 = ANY (array_agg(language_id)));
    END IF;
END
$$
