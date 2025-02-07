CREATE TABLE comment_aggregates (
    comment_id integer NOT NULL,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    published timestamp with time zone NOT NULL DEFAULT now(),
    child_count integer NOT NULL DEFAULT 0,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    controversy_rank double precision NOT NULL DEFAULT 0,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0
);

INSERT INTO comment_aggregates
SELECT
    id AS comment_id,
    score,
    upvotes,
    downvotes,
    published,
    child_count,
    hot_rank,
    controversy_rank,
    report_count,
    unresolved_report_count
FROM
    comment;

ALTER TABLE comment
    DROP COLUMN score,
    DROP COLUMN upvotes,
    DROP COLUMN downvotes,
    DROP COLUMN child_count,
    DROP COLUMN hot_rank,
    DROP COLUMN controversy_rank,
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

