-- Adding report_count and unresolved_report_count
-- to the post and comment aggregate tables
ALTER TABLE post_aggregates
    ADD COLUMN report_count bigint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count bigint NOT NULL DEFAULT 0;

ALTER TABLE comment_aggregates
    ADD COLUMN report_count bigint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count bigint NOT NULL DEFAULT 0;

