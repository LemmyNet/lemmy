
CREATE TABLE report_combined (
    id serial PRIMARY KEY,
    published timestamptz not null,
    post_report_id int REFERENCES post_report ON UPDATE CASCADE ON DELETE CASCADE,
    comment_report_id int REFERENCES comment_report ON UPDATE CASCADE ON DELETE CASCADE,
    UNIQUE (post_report_id, comment_report_id)
);

CREATE INDEX idx_report_combined_published on report_combined (published desc);

-- TODO do history update
-- TODO do triggers in replaceable schema

