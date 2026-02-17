ALTER TABLE comment_report
    ALTER COLUMN creator_id DROP NOT NULL;

ALTER TABLE comment_report
    ADD COLUMN creator_site_id int NOT NULL REFERENCES site (id);

ALTER TABLE post_report
    ALTER COLUMN creator_id DROP NOT NULL;

ALTER TABLE post_report
    ADD COLUMN creator_site_id int NOT NULL REFERENCES site (id);

ALTER TABLE community_report
    ALTER COLUMN creator_id DROP NOT NULL;

ALTER TABLE community_report
    ADD COLUMN creator_site_id int NOT NULL REFERENCES site (id);

ALTER TABLE private_message_report
    ALTER COLUMN creator_id DROP NOT NULL;

ALTER TABLE private_message_report
    ADD COLUMN creator_site_id int NOT NULL REFERENCES site (id);

