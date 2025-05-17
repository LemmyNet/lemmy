ALTER TABLE post_report
    ADD COLUMN violates_instance_rules bool NOT NULL DEFAULT FALSE;

ALTER TABLE comment_report
    ADD COLUMN violates_instance_rules bool NOT NULL DEFAULT FALSE;

