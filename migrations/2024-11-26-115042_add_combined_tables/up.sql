CREATE TABLE report_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    post_report_id int REFERENCES post_report ON UPDATE CASCADE ON DELETE CASCADE,
    comment_report_id int REFERENCES comment_report ON UPDATE CASCADE ON DELETE CASCADE,
    private_message_report_id int REFERENCES private_message_report ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE (post_report_id, comment_report_id, private_message_report_id)
);

CREATE INDEX idx_report_combined_published ON report_combined (published DESC);

-- TODO do history update
-- TODO do triggers in replaceable schema
