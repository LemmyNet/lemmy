ALTER TABLE admin_allow_instance RENAME published_at TO published;

ALTER TABLE admin_block_instance RENAME COLUMN expires_at TO expires;

ALTER TABLE admin_block_instance RENAME COLUMN published_at TO published;

ALTER TABLE admin_purge_comment RENAME COLUMN published_at TO published;

ALTER TABLE admin_purge_community RENAME COLUMN published_at TO published;

ALTER TABLE admin_purge_person RENAME COLUMN published_at TO published;

ALTER TABLE admin_purge_post RENAME COLUMN published_at TO published;

ALTER TABLE captcha_answer RENAME COLUMN published_at TO published;

ALTER TABLE comment RENAME COLUMN published_at TO published;

ALTER TABLE comment RENAME COLUMN updated_at TO updated;

ALTER TABLE comment_actions RENAME COLUMN voted_at TO liked;

ALTER TABLE comment_actions RENAME COLUMN saved_at TO saved;

ALTER TABLE comment_reply RENAME COLUMN published_at TO published;

ALTER TABLE comment_report RENAME COLUMN published_at TO published;

ALTER TABLE comment_report RENAME COLUMN updated_at TO updated;

ALTER TABLE community RENAME COLUMN published_at TO published;

ALTER TABLE community RENAME COLUMN updated_at TO updated;

ALTER TABLE community_actions RENAME COLUMN followed_at TO followed;

ALTER TABLE community_actions RENAME COLUMN blocked_at TO blocked;

ALTER TABLE community_actions RENAME COLUMN became_moderator_at TO became_moderator;

ALTER TABLE community_actions RENAME COLUMN received_ban_at TO received_ban;

ALTER TABLE community_actions RENAME COLUMN ban_expires_at TO ban_expires;

ALTER TABLE community_report RENAME COLUMN published_at TO published;

ALTER TABLE community_report RENAME COLUMN updated_at TO updated;

ALTER TABLE custom_emoji RENAME COLUMN published_at TO published;

ALTER TABLE custom_emoji RENAME COLUMN updated_at TO updated;

ALTER TABLE email_verification RENAME COLUMN published_at TO published;

ALTER TABLE federation_allowlist RENAME COLUMN published_at TO published;

ALTER TABLE federation_allowlist RENAME COLUMN updated_at TO updated;

ALTER TABLE federation_blocklist RENAME COLUMN published_at TO published;

ALTER TABLE federation_blocklist RENAME COLUMN updated_at TO updated;

ALTER TABLE federation_blocklist RENAME COLUMN expires_at TO expires;

ALTER TABLE federation_queue_state RENAME COLUMN last_retry_at TO last_retry;

ALTER TABLE federation_queue_state RENAME COLUMN last_successful_published_time_at TO last_successful_published_time;

ALTER TABLE inbox_combined RENAME COLUMN published_at TO published;

ALTER TABLE instance RENAME COLUMN published_at TO published;

ALTER TABLE instance RENAME COLUMN updated_at TO updated;

ALTER TABLE instance_actions RENAME COLUMN blocked_at TO blocked;

ALTER TABLE instance_actions RENAME COLUMN received_ban_at TO received_ban;

ALTER TABLE instance_actions RENAME COLUMN ban_expires_at TO ban_expires;

ALTER TABLE local_image RENAME COLUMN published_at TO published;

ALTER TABLE local_site RENAME COLUMN published_at TO published;

ALTER TABLE local_site RENAME COLUMN updated_at TO updated;

ALTER TABLE local_site_rate_limit RENAME COLUMN published_at TO published;

ALTER TABLE local_site_rate_limit RENAME COLUMN updated_at TO updated;

ALTER TABLE local_site_url_blocklist RENAME COLUMN published_at TO published;

ALTER TABLE local_site_url_blocklist RENAME COLUMN updated_at TO updated;

ALTER TABLE local_user RENAME COLUMN last_donation_notification_at TO last_donation_notification;

ALTER TABLE login_token RENAME COLUMN published_at TO published;

ALTER TABLE mod_add RENAME COLUMN published_at TO published;

ALTER TABLE mod_add_community RENAME COLUMN published_at TO published;

ALTER TABLE mod_ban RENAME COLUMN published_at TO published;

ALTER TABLE mod_ban RENAME COLUMN expires_at TO expires;

ALTER TABLE mod_ban_from_community RENAME COLUMN published_at TO published;

ALTER TABLE mod_ban_from_community RENAME COLUMN expires_at TO expires;

ALTER TABLE mod_change_community_visibility RENAME COLUMN published_at TO published;

ALTER TABLE mod_feature_post RENAME COLUMN published_at TO published;

ALTER TABLE mod_lock_post RENAME COLUMN published_at TO published;

ALTER TABLE mod_remove_comment RENAME COLUMN published_at TO published;

ALTER TABLE mod_remove_community RENAME COLUMN published_at TO published;

ALTER TABLE mod_remove_post RENAME COLUMN published_at TO published;

ALTER TABLE mod_transfer_community RENAME COLUMN published_at TO published;

ALTER TABLE modlog_combined RENAME COLUMN published_at TO published;

ALTER TABLE oauth_account RENAME COLUMN published_at TO published;

ALTER TABLE oauth_account RENAME COLUMN updated_at TO updated;

ALTER TABLE oauth_provider RENAME COLUMN published_at TO published;

ALTER TABLE oauth_provider RENAME COLUMN updated_at TO updated;

ALTER TABLE password_reset_request RENAME COLUMN published_at TO published;

ALTER TABLE person RENAME COLUMN published_at TO published;

ALTER TABLE person RENAME COLUMN updated_at TO updated;

ALTER TABLE person_actions RENAME COLUMN followed_at TO followed;

ALTER TABLE person_actions RENAME COLUMN blocked_at TO blocked;

ALTER TABLE person_ban RENAME COLUMN published_at TO published;

ALTER TABLE person_comment_mention RENAME COLUMN published_at TO published;

ALTER TABLE person_content_combined RENAME COLUMN published_at TO published;

ALTER TABLE person_liked_combined RENAME COLUMN voted_at TO liked;

ALTER TABLE person_post_mention RENAME COLUMN published_at TO published;

ALTER TABLE person_saved_combined RENAME COLUMN saved_at TO saved;

ALTER TABLE post RENAME COLUMN published_at TO published;

ALTER TABLE post RENAME COLUMN updated_at TO updated;

ALTER TABLE post RENAME COLUMN scheduled_publish_time_at TO scheduled_publish_time;

ALTER TABLE post RENAME COLUMN newest_comment_time_at TO newest_comment_time;

ALTER TABLE post RENAME COLUMN newest_comment_time_necro_at TO newest_comment_time_necro;

ALTER TABLE post_actions RENAME COLUMN read_at TO read;

ALTER TABLE post_actions RENAME COLUMN read_comments_at TO read_comments;

ALTER TABLE post_actions RENAME COLUMN saved_at TO saved;

ALTER TABLE post_actions RENAME COLUMN voted_at TO liked;

ALTER TABLE post_actions RENAME COLUMN hidden_at TO hidden;

ALTER TABLE post_report RENAME COLUMN published_at TO published;

ALTER TABLE post_report RENAME COLUMN updated_at TO updated;

ALTER TABLE post_tag RENAME COLUMN published_at TO published;

ALTER TABLE private_message RENAME COLUMN published_at TO published;

ALTER TABLE private_message RENAME COLUMN updated_at TO updated;

ALTER TABLE private_message_report RENAME COLUMN published_at TO published;

ALTER TABLE private_message_report RENAME COLUMN updated_at TO updated;

ALTER TABLE received_activity RENAME COLUMN published_at TO published;

ALTER TABLE registration_application RENAME COLUMN published_at TO published;

ALTER TABLE remote_image RENAME COLUMN published_at TO published;

ALTER TABLE report_combined RENAME COLUMN published_at TO published;

ALTER TABLE search_combined RENAME COLUMN published_at TO published;

ALTER TABLE sent_activity RENAME COLUMN published_at TO published;

ALTER TABLE site RENAME COLUMN published_at TO published;

ALTER TABLE site RENAME COLUMN updated_at TO updated;

ALTER TABLE tag RENAME COLUMN published_at TO published;

ALTER TABLE tag RENAME COLUMN updated_at TO updated;

ALTER TABLE tagline RENAME COLUMN published_at TO published;

ALTER TABLE tagline RENAME COLUMN updated_at TO updated;

