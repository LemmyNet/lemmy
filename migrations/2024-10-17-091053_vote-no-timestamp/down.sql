-- make published not null again and add default
ALTER TABLE post_like
    ALTER COLUMN published SET NOT NULL;

ALTER TABLE post_like
    ALTER COLUMN published SET DEFAULT now();

ALTER TABLE comment_like
    ALTER COLUMN published DROP NOT NULL;

ALTER TABLE comment_like
    ALTER COLUMN published SET DEFAULT now();

-- restore comment_like.post_id
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

