CREATE TABLE community_report (
    id serial PRIMARY KEY,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    community_id int REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    original_community_title text NOT NULL,
    original_community_description text NOT NULL,
    original_community_icon text NOT NULL,
    original_community_banner text NOT NULL,
    reason text NOT NULL,
    resolved bool NOT NULL DEFAULT FALSE,
    resolver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz NULL,
    UNIQUE (community_id, creator_id)
);

CREATE INDEX idx_community_report_published ON community_report (published DESC);

