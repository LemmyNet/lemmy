DELETE FROM report_combined
WHERE community_report_id IS NOT NULL;

ALTER TABLE report_combined
    DROP CONSTRAINT report_combined_check,
    ADD CHECK (num_nonnulls (post_report_id, comment_report_id, private_message_report_id) = 1),
    DROP COLUMN community_report_id;

DROP TABLE community_report CASCADE;

ALTER TABLE community_aggregates
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

