DROP INDEX idx_report_combined_published_asc;

ALTER TABLE report_combined
    DROP COLUMN resolved;

CREATE INDEX idx_report_combined_published_asc ON report_combined (reverse_timestamp_sort (published_at) DESC, id DESC);

