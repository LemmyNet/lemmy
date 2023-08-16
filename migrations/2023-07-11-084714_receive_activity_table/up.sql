-- outgoing activities, need to be stored to be later server over http
-- we change data column from jsonb to json for decreased size
-- https://stackoverflow.com/a/22910602
CREATE TABLE sent_activity (
    id bigserial PRIMARY KEY,
    ap_id text UNIQUE NOT NULL,
    data json NOT NULL,
    sensitive boolean NOT NULL,
    published timestamp NOT NULL DEFAULT now()
);

-- incoming activities, we only need the id to avoid processing the same activity multiple times
CREATE TABLE received_activity (
    id bigserial PRIMARY KEY,
    ap_id text UNIQUE NOT NULL,
    published timestamp NOT NULL DEFAULT now()
);

-- copy sent activities to new table. only copy last 100k for faster migration
INSERT INTO sent_activity (ap_id, data, sensitive, published)
SELECT
    ap_id,
    data,
    sensitive,
    published
FROM
    activity
WHERE
    local = TRUE
ORDER BY
    id DESC
LIMIT 100000;

-- copy received activities to new table. only last 1m for faster migration
INSERT INTO received_activity (ap_id, published)
SELECT
    ap_id,
    published
FROM
    activity
WHERE
    local = FALSE
ORDER BY
    id DESC
LIMIT 1000000;

DROP TABLE activity;

