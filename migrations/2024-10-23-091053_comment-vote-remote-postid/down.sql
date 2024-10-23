ALTER TABLE comment_like
    ADD COLUMN post_id int;

UPDATE
    comment_like
SET
    post_id = comment.post_id
FROM
    comment
WHERE
    comment_id = comment.id;

ALTER TABLE comment_like
    ALTER COLUMN post_id SET NOT NULL;

