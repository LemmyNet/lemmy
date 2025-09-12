ALTER TABLE notification
    ADD COLUMN mod_remove_comment_id int REFERENCES mod_remove_comment ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TYPE enum NotificationTypeEnum
    ADD value 'ModAction';

ALTER TYPE enum NotificationTypeEnum
    ADD value 'RevertModAction';

