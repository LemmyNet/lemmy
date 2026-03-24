ALTER TABLE local_site RENAME COLUMN disable_email_notifications TO email_notifications_disabled;

ALTER TABLE local_site RENAME COLUMN require_email_verification TO email_verification_required;

ALTER TABLE local_site RENAME COLUMN disallow_nsfw_content TO nsfw_content_disallowed;

ALTER TABLE local_user RENAME COLUMN enable_animated_images TO animated_images_enabled;

ALTER TABLE local_user RENAME COLUMN enable_private_messages TO private_messages_enabled;

