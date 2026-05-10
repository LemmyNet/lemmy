ALTER TABLE local_site RENAME COLUMN email_notifications_disabled TO disable_email_notifications;

ALTER TABLE local_site RENAME COLUMN email_verification_required TO require_email_verification;

ALTER TABLE local_site RENAME COLUMN nsfw_content_disallowed TO disallow_nsfw_content;

ALTER TABLE local_user RENAME COLUMN animated_images_enabled TO enable_animated_images;

ALTER TABLE local_user RENAME COLUMN private_messages_enabled TO enable_private_messages;

