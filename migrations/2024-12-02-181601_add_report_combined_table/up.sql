-- Creates combined tables for the following:
--
-- Reports: (comment, post, and private_message)
-- Inbox: (Comment replies, post replies, comment mentions, post mentions, private messages)
-- Profile: (Posts and Comments)
-- Modlog: (lots of types)
-- Search: (community, post, comment, user, url)
-- TODO not sure about these two:
-- Home: (comment, post)
-- Community: (comment, post)
CREATE TABLE report_combined (
    id serial PRIMARY KEY,
    published timestamptz NOT NULL,
    post_report_id int UNIQUE REFERENCES post_report ON UPDATE CASCADE ON DELETE CASCADE,
    comment_report_id int UNIQUE REFERENCES comment_report ON UPDATE CASCADE ON DELETE CASCADE,
    private_message_report_id int UNIQUE REFERENCES private_message_report ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK ((post_report_id IS NOT NULL)::integer + (comment_report_id IS NOT NULL)::integer + (private_message_report_id IS NOT NULL)::integer = 1)
);

CREATE INDEX idx_report_combined_published ON report_combined (published DESC, id DESC);

CREATE INDEX idx_report_combined_published_asc ON report_combined (reverse_timestamp_sort (published) DESC, id DESC);

-- Updating the history
INSERT INTO report_combined (published, post_report_id)
SELECT
    published,
    id
FROM
    post_report;

INSERT INTO report_combined (published, comment_report_id)
SELECT
    published,
    id
FROM
    comment_report;

INSERT INTO report_combined (published, private_message_report_id)
SELECT
    published,
    id
FROM
    private_message_report;

