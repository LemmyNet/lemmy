ALTER TABLE comment_actions
    ADD COLUMN post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE;

UPDATE
    comment_actions
SET
    post_id = (
        SELECT
            post_id
        FROM
            comment
        WHERE
            comment.id = comment_actions.comment_id)
WHERE
    liked IS NOT NULL;

