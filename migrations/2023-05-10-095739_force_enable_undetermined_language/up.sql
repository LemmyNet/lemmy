-- force enable undetermined language for all users
INSERT INTO local_user_language (local_user_id, language_id)
SELECT
    id,
    0
FROM
    local_user
ON CONFLICT (local_user_id,
    language_id)
    DO NOTHING;

