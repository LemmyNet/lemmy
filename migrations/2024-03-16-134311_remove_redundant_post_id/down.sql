ALTER TABLE comment_like
    ADD COLUMN post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE;

UPDATE
    comment_like
SET
    post_id = (
        SELECT
            post_id
        FROM
            comment
        WHERE
            comment.id = comment_like.comment_id);

ALTER TABLE comment_like
    ALTER COLUMN post_id SET NOT NULL;

