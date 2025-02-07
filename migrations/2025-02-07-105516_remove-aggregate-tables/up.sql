ALTER TABLE comment
    ADD COLUMN score bigint NOT NULL DEFAULT 0,
    ADD COLUMN upvotes bigint NOT NULL DEFAULT 0,
    ADD COLUMN downvotes bigint NOT NULL DEFAULT 0,
    ADD COLUMN child_count integer NOT NULL DEFAULT 0,
    ADD COLUMN hot_rank double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN controversy_rank double precision NOT NULL DEFAULT 0,
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

UPDATE
    comment
SET
    score = ca.score,
    upvotes = ca.upvotes,
    downvotes = ca.downvotes,
    child_count = ca.child_count,
    hot_rank = ca.hot_rank,
    controversy_rank = ca.controversy_rank,
    report_count = ca.report_count,
    unresolved_report_count = ca.unresolved_report_count
FROM
    comment_aggregates AS ca
WHERE
    comment.id = ca.comment_id;

DROP TABLE comment_aggregates;

