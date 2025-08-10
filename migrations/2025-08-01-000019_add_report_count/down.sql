ALTER TABLE post_aggregates
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

ALTER TABLE comment_aggregates
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

