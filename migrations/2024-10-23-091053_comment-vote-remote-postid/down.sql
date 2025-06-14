ALTER TABLE comment_like
    ADD COLUMN post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL;

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

CREATE INDEX idx_comment_like_post ON comment_like (post_id);

