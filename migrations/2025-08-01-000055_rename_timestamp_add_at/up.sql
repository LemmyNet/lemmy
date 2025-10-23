ALTER TABLE admin_allow_instance RENAME COLUMN published TO published_at;

ALTER TABLE admin_block_instance RENAME COLUMN expires TO expires_at;

ALTER TABLE admin_block_instance RENAME COLUMN published TO published_at;

ALTER TABLE admin_purge_comment RENAME COLUMN published TO published_at;

ALTER TABLE admin_purge_community RENAME COLUMN published TO published_at;

ALTER TABLE admin_purge_person RENAME COLUMN published TO published_at;

ALTER TABLE admin_purge_post RENAME COLUMN published TO published_at;

ALTER TABLE captcha_answer RENAME COLUMN published TO published_at;

ALTER TABLE comment RENAME COLUMN published TO published_at;

ALTER TABLE comment RENAME COLUMN updated TO updated_at;

ALTER TABLE comment_actions RENAME COLUMN liked TO voted_at;

ALTER TABLE comment_actions RENAME COLUMN saved TO saved_at;

ALTER TABLE comment_reply RENAME COLUMN published TO published_at;

ALTER TABLE comment_report RENAME COLUMN published TO published_at;

ALTER TABLE comment_report RENAME COLUMN updated TO updated_at;

ALTER TABLE community RENAME COLUMN published TO published_at;

ALTER TABLE community RENAME COLUMN updated TO updated_at;

ALTER TABLE community_actions RENAME COLUMN followed TO followed_at;

ALTER TABLE community_actions RENAME COLUMN blocked TO blocked_at;

ALTER TABLE community_actions RENAME COLUMN became_moderator TO became_moderator_at;

ALTER TABLE community_actions RENAME COLUMN received_ban TO received_ban_at;

ALTER TABLE community_actions RENAME COLUMN ban_expires TO ban_expires_at;

ALTER TABLE community_report RENAME COLUMN published TO published_at;

ALTER TABLE community_report RENAME COLUMN updated TO updated_at;

ALTER TABLE custom_emoji RENAME COLUMN published TO published_at;

ALTER TABLE custom_emoji RENAME COLUMN updated TO updated_at;

ALTER TABLE email_verification RENAME COLUMN published TO published_at;

ALTER TABLE federation_allowlist RENAME COLUMN published TO published_at;

ALTER TABLE federation_allowlist RENAME COLUMN updated TO updated_at;

ALTER TABLE federation_blocklist RENAME COLUMN published TO published_at;

ALTER TABLE federation_blocklist RENAME COLUMN updated TO updated_at;

ALTER TABLE federation_blocklist RENAME COLUMN expires TO expires_at;

ALTER TABLE federation_queue_state RENAME COLUMN last_retry TO last_retry_at;

ALTER TABLE federation_queue_state RENAME COLUMN last_successful_published_time TO last_successful_published_time_at;

ALTER TABLE inbox_combined RENAME COLUMN published TO published_at;

ALTER TABLE instance RENAME COLUMN published TO published_at;

ALTER TABLE instance RENAME COLUMN updated TO updated_at;

ALTER TABLE instance_actions RENAME COLUMN blocked TO blocked_at;

ALTER TABLE instance_actions RENAME COLUMN received_ban TO received_ban_at;

ALTER TABLE instance_actions RENAME COLUMN ban_expires TO ban_expires_at;

ALTER TABLE local_image RENAME COLUMN published TO published_at;

ALTER TABLE local_site RENAME COLUMN published TO published_at;

ALTER TABLE local_site RENAME COLUMN updated TO updated_at;

ALTER TABLE local_site_rate_limit RENAME COLUMN published TO published_at;

ALTER TABLE local_site_rate_limit RENAME COLUMN updated TO updated_at;

ALTER TABLE local_site_url_blocklist RENAME COLUMN published TO published_at;

ALTER TABLE local_site_url_blocklist RENAME COLUMN updated TO updated_at;

ALTER TABLE local_user RENAME COLUMN last_donation_notification TO last_donation_notification_at;

ALTER TABLE login_token RENAME COLUMN published TO published_at;

ALTER TABLE mod_add RENAME COLUMN published TO published_at;

ALTER TABLE mod_add_community RENAME COLUMN published TO published_at;

ALTER TABLE mod_ban RENAME COLUMN published TO published_at;

ALTER TABLE mod_ban RENAME COLUMN expires TO expires_at;

ALTER TABLE mod_ban_from_community RENAME COLUMN published TO published_at;

ALTER TABLE mod_ban_from_community RENAME COLUMN expires TO expires_at;

ALTER TABLE mod_change_community_visibility RENAME COLUMN published TO published_at;

ALTER TABLE mod_feature_post RENAME COLUMN published TO published_at;

ALTER TABLE mod_lock_post RENAME COLUMN published TO published_at;

ALTER TABLE mod_remove_comment RENAME COLUMN published TO published_at;

ALTER TABLE mod_remove_community RENAME COLUMN published TO published_at;

ALTER TABLE mod_remove_post RENAME COLUMN published TO published_at;

ALTER TABLE mod_transfer_community RENAME COLUMN published TO published_at;

ALTER TABLE modlog_combined RENAME COLUMN published TO published_at;

ALTER TABLE oauth_account RENAME COLUMN published TO published_at;

ALTER TABLE oauth_account RENAME COLUMN updated TO updated_at;

ALTER TABLE oauth_provider RENAME COLUMN published TO published_at;

ALTER TABLE oauth_provider RENAME COLUMN updated TO updated_at;

ALTER TABLE password_reset_request RENAME COLUMN published TO published_at;

ALTER TABLE person RENAME COLUMN published TO published_at;

ALTER TABLE person RENAME COLUMN updated TO updated_at;

ALTER TABLE person_actions RENAME COLUMN followed TO followed_at;

ALTER TABLE person_actions RENAME COLUMN blocked TO blocked_at;

ALTER TABLE person_ban RENAME COLUMN published TO published_at;

ALTER TABLE person_comment_mention RENAME COLUMN published TO published_at;

ALTER TABLE person_content_combined RENAME COLUMN published TO published_at;

ALTER TABLE person_liked_combined RENAME COLUMN liked TO voted_at;

ALTER TABLE person_post_mention RENAME COLUMN published TO published_at;

ALTER TABLE person_saved_combined RENAME COLUMN saved TO saved_at;

ALTER TABLE post RENAME COLUMN published TO published_at;

ALTER TABLE post RENAME COLUMN updated TO updated_at;

ALTER TABLE post RENAME COLUMN scheduled_publish_time TO scheduled_publish_time_at;

ALTER TABLE post RENAME COLUMN newest_comment_time TO newest_comment_time_at;

ALTER TABLE post RENAME COLUMN newest_comment_time_necro TO newest_comment_time_necro_at;

ALTER TABLE post_actions RENAME COLUMN read TO read_at;

ALTER TABLE post_actions RENAME COLUMN read_comments TO read_comments_at;

ALTER TABLE post_actions RENAME COLUMN saved TO saved_at;

ALTER TABLE post_actions RENAME COLUMN liked TO voted_at;

ALTER TABLE post_actions RENAME COLUMN hidden TO hidden_at;

ALTER TABLE post_report RENAME COLUMN published TO published_at;

ALTER TABLE post_report RENAME COLUMN updated TO updated_at;

ALTER TABLE post_tag RENAME COLUMN published TO published_at;

ALTER TABLE private_message RENAME COLUMN published TO published_at;

ALTER TABLE private_message RENAME COLUMN updated TO updated_at;

ALTER TABLE private_message_report RENAME COLUMN published TO published_at;

ALTER TABLE private_message_report RENAME COLUMN updated TO updated_at;

ALTER TABLE received_activity RENAME COLUMN published TO published_at;

ALTER TABLE registration_application RENAME COLUMN published TO published_at;

ALTER TABLE remote_image RENAME COLUMN published TO published_at;

ALTER TABLE report_combined RENAME COLUMN published TO published_at;

ALTER TABLE search_combined RENAME COLUMN published TO published_at;

ALTER TABLE sent_activity RENAME COLUMN published TO published_at;

ALTER TABLE site RENAME COLUMN published TO published_at;

ALTER TABLE site RENAME COLUMN updated TO updated_at;

ALTER TABLE tag RENAME COLUMN published TO published_at;

ALTER TABLE tag RENAME COLUMN updated TO updated_at;

ALTER TABLE tagline RENAME COLUMN published TO published_at;

ALTER TABLE tagline RENAME COLUMN updated TO updated_at;

