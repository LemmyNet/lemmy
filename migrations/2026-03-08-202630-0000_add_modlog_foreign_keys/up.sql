ALTER TABLE modlog
    ADD CONSTRAINT modlog_mod_fkey FOREIGN KEY (mod_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_target_person_fkey FOREIGN KEY (target_person_id) REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_target_community_fkey FOREIGN KEY (target_community_id) REFERENCES community ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_target_post_fkey FOREIGN KEY (target_post_id) REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_target_comment_fkey FOREIGN KEY (target_comment_id) REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_target_instance_fkey FOREIGN KEY (target_instance_id) REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_bulk_action_parent_fkey FOREIGN KEY (bulk_action_parent_id) REFERENCES modlog ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX idx_modlog_kind ON modlog (kind);

CREATE INDEX idx_modlog_mod ON modlog (mod_id);

CREATE INDEX idx_modlog_target_person ON modlog (target_person_id)
WHERE
    target_person_id IS NOT NULL;

CREATE INDEX idx_modlog_target_community ON modlog (target_community_id)
WHERE
    target_community_id IS NOT NULL;

CREATE INDEX idx_modlog_target_post ON modlog (target_post_id)
WHERE
    target_post_id IS NOT NULL;

CREATE INDEX idx_modlog_target_comment ON modlog (target_comment_id)
WHERE
    target_comment_id IS NOT NULL;

CREATE INDEX idx_modlog_target_instance ON modlog (target_instance_id)
WHERE
    target_instance_id IS NOT NULL;

CREATE INDEX idx_modlog_published_id ON modlog (published_at DESC, id DESC);

