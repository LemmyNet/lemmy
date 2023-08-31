ALTER TABLE private_message
    ADD COLUMN ap_id character varying(255) NOT NULL DEFAULT 'http://fake.com', -- This needs to be checked and updated in code, building from the site url if local
    ADD COLUMN local boolean NOT NULL DEFAULT TRUE;

DROP MATERIALIZED VIEW private_message_mview;

DROP VIEW private_message_view;

CREATE VIEW private_message_view AS
SELECT
    pm.*,
    u.name AS creator_name,
    u.avatar AS creator_avatar,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u2.name AS recipient_name,
    u2.avatar AS recipient_avatar,
    u2.actor_id AS recipient_actor_id,
    u2.local AS recipient_local
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

