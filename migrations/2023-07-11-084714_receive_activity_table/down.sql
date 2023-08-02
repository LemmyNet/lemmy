CREATE TABLE activity (
    id serial PRIMARY KEY,
    data jsonb NOT NULL,
    local boolean NOT NULL DEFAULT TRUE,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp,
    ap_id text NOT NULL,
    sensitive boolean NOT NULL DEFAULT TRUE
);

INSERT INTO activity (ap_id, data, sensitive, published)
SELECT
    ap_id,
    data,
    sensitive,
    published
FROM
    sent_activity
ORDER BY
    id DESC
LIMIT 100000;

-- We cant copy received_activity entries back into activities table because we dont have data
-- which is mandatory.
DROP TABLE sent_activity;

DROP TABLE received_activity;

