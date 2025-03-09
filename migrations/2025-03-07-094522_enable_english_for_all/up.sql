-- enable english for all users
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

