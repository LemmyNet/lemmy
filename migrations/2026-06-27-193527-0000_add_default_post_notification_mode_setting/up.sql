ALTER TABLE local_user
    ADD COLUMN default_post_notifications_mode public.post_notifications_mode_enum NOT NULL DEFAULT 'RepliesAndMentions';

ALTER TABLE local_site
    ADD COLUMN default_post_notifications_mode public.post_notifications_mode_enum NOT NULL DEFAULT 'RepliesAndMentions';

