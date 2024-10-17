ALTER TABLE post_like
    ADD COLUMN published timestamptz NOT NULL DEFAULT now();

ALTER TABLE comment_like
    ADD COLUMN published timestamptz NOT NULL DEFAULT now();

