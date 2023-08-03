CREATE TABLE private_message_report (
    id serial PRIMARY KEY,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- user reporting comment
    private_message_id int REFERENCES private_message ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, -- comment being reported
    original_pm_text text NOT NULL,
    reason text NOT NULL,
    resolved bool NOT NULL DEFAULT FALSE,
    resolver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE, -- user resolving report
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp NULL,
    UNIQUE (private_message_id, creator_id) -- users should only be able to report a pm once
);

