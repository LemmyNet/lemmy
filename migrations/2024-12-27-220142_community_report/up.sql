CREATE TABLE community_report (
    id serial PRIMARY KEY,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    original_community_name text NOT NULL,
    original_community_title text NOT NULL,
    original_community_description text,
    original_community_sidebar text,
    original_community_icon text,
    original_community_banner text,
    reason text NOT NULL,
    resolved bool NOT NULL DEFAULT FALSE,
    resolver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz NULL,
    UNIQUE (community_id, creator_id)
);

CREATE INDEX idx_community_report_published ON community_report (published DESC);

ALTER TABLE report_combined
    ADD COLUMN community_report_id int UNIQUE REFERENCES community_report ON UPDATE CASCADE ON DELETE CASCADE,
    DROP CONSTRAINT report_combined_check,
    ADD CHECK (num_nonnulls (post_report_id, comment_report_id, private_message_report_id, community_report_id) = 1);

ALTER TABLE community_aggregates
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

