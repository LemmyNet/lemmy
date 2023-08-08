DROP MATERIALIZED VIEW private_message_mview;

DROP VIEW private_message_view;

ALTER TABLE private_message
    DROP COLUMN ap_id,
    DROP COLUMN local;

CREATE VIEW private_message_view AS
SELECT
    pm.*,
    u.name AS creator_name,
    u.avatar AS creator_avatar,
    u2.name AS recipient_name,
    u2.avatar AS recipient_avatar
FROM
    private_message pm
    INNER JOIN user_ u ON u.id = pm.creator_id
    INNER JOIN user_ u2 ON u2.id = pm.recipient_id;

CREATE MATERIALIZED VIEW private_message_mview AS
SELECT
    *
FROM
    private_message_view;

CREATE UNIQUE INDEX idx_private_message_mview_id ON private_message_mview (id);

