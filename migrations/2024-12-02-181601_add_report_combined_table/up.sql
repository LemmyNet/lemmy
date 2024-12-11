-- Creates combined tables for
-- Reports: (comment, post, and private_message)
CREATE TABLE report_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    post_report_id int UNIQUE REFERENCES post_report ON UPDATE CASCADE ON DELETE CASCADE,
    comment_report_id int UNIQUE REFERENCES comment_report ON UPDATE CASCADE ON DELETE CASCADE,
    private_message_report_id int UNIQUE REFERENCES private_message_report ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (post_report_id, comment_report_id, private_message_report_id) = 1)
);

CREATE INDEX idx_report_combined_published ON report_combined (published DESC, id DESC);

CREATE INDEX idx_report_combined_published_asc ON report_combined (reverse_timestamp_sort (published) DESC, id DESC);

-- Updating the history
INSERT INTO report_combined (published, post_report_id, comment_report_id, private_message_report_id)
SELECT
    published,
    id,
    NULL::int,
    NULL::int
FROM
    post_report
UNION ALL
SELECT
    published,
    NULL::int,
    id,
    NULL::int
FROM
    comment_report
UNION ALL
SELECT
    published,
    NULL::int,
    NULL::int,
    id
FROM
    private_message_report;

