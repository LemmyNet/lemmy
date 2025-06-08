alter table admin_allow_instance rename column published to published_at;

alter table admin_block_instance rename column expires to expires_at;
alter table admin_block_instance rename column published to published_at;

alter table admin_purge_comment rename column published to published_at;
alter table admin_purge_community rename column published to published_at;
alter table admin_purge_person rename column published to published_at;
alter table admin_purge_post rename column published to published_at;

alter table captcha_answer rename column published to published_at;

alter table comment rename column published to published_at;
alter table comment rename column updated to updated_at;

alter table comment_actions rename column liked to liked_at;
alter table comment_actions rename column saved to saved_at;

alter table comment_reply rename column published to published_at;

alter table comment_report rename column published to published_at;
alter table comment_report rename column updated to updated_at;

alter table community rename column published to published_at;
alter table community rename column updated to updated_at;

alter table community_actions rename column followed to followed_at;
alter table community_actions rename column blocked to blocked_at;
alter table community_actions rename column became_moderator to became_moderator_at;
alter table community_actions rename column received_ban to received_ban_at;
alter table community_actions rename column ban_expires to ban_expires_at;

alter table community_report rename column published to published_at;
alter table community_report rename column updated to updated_at;

alter table custom_emoji rename column published to published_at;
alter table custom_emoji rename column updated to updated_at;

alter table email_verification rename column published to published_at;

alter table federation_allowlist rename column published to published_at;
alter table federation_allowlist rename column updated to updated_at;

alter table federation_blocklist rename column published to published_at;
alter table federation_blocklist rename column updated to updated_at;
alter table federation_blocklist rename column expires to expires_at;


alter table federation_queue_state rename column last_retry to last_retry_at;
alter table federation_queue_state rename column last_successful_published_time to last_successful_published_time_at;

alter table inbox_combined rename column published to published_at;

alter table instance rename column published to published_at;
alter table instance rename column updated to updated_at;

alter table instance_actions rename column blocked to blocked_at;
alter table instance_actions rename column received_ban to received_ban_at;
alter table instance_actions rename column ban_expires to ban_expires_at;

alter table local_image rename column published to published_at;

alter table local_site rename column published to published_at;
alter table local_site rename column updated to updated_at;

alter table local_site_rate_limit rename column published to published_at;
alter table local_site_rate_limit rename column updated to updated_at;

alter table local_site_url_blocklist rename column published to published_at;
alter table local_site_url_blocklist rename column updated to updated_at;

alter table local_user rename column last_donation_notification to last_donation_notification_at;

alter table login_token rename column published to published_at;

alter table mod_add rename column published to published_at;
alter table mod_add_community rename column published to published_at;

alter table mod_ban rename column published to published_at;
alter table mod_ban rename column expires to expires_at;

alter table mod_ban_from_community rename column published to published_at;
alter table mod_ban_from_community rename column expires to expires_at;

alter table mod_change_community_visibility rename column published to published_at;

alter table mod_feature_post rename column published to published_at;
alter table mod_lock_post rename column published to published_at;

alter table mod_remove_comment rename column published to published_at;
alter table mod_remove_community rename column published to published_at;
alter table mod_remove_post rename column published to published_at;
alter table mod_transfer_community rename column published to published_at;

alter table modlog_combined rename column published to published_at;

alter table oauth_account rename column published to published_at;
alter table oauth_account rename column updated to updated_at;

alter table oauth_provider rename column published to published_at;
alter table oauth_provider rename column updated to updated_at;

alter table password_reset_request rename column published to published_at;

alter table person rename column published to published_at;
alter table person rename column updated to updated_at;

alter table person_actions rename column followed to followed_at;
alter table person_actions rename column blocked to blocked_at;

alter table person_ban rename column published to published_at;

alter table person_comment_mention rename column published to published_at;

alter table person_content_combined rename column published to published_at;

alter table person_liked_combined rename column liked to liked_at;

alter table person_post_mention rename column published to published_at;

alter table person_saved_combined rename column saved to saved_at;

alter table post rename column published to published_at;
alter table post rename column updated to updated_at;
alter table post rename column scheduled_publish_time to scheduled_publish_time_at;
alter table post rename column newest_comment_time to newest_comment_time_at;
alter table post rename column newest_comment_time_necro to newest_comment_time_necro_at;

alter table post_actions rename column read to read_at;
alter table post_actions rename column read_comments to read_comments_at;
alter table post_actions rename column saved to saved_at;
alter table post_actions rename column liked to liked_at;
alter table post_actions rename column hidden to hidden_at;

alter table post_report rename column published to published_at;
alter table post_report rename column updated to updated_at;

alter table post_tag rename column published to published_at;

alter table private_message rename column published to published_at;
alter table private_message rename column updated to updated_at;

alter table private_message_report rename column published to published_at;
alter table private_message_report rename column updated to updated_at;

alter table received_activity rename column published to published_at;

alter table registration_application rename column published to published_at;

alter table remote_image rename column published to published_at;

alter table report_combined rename column published to published_at;
alter table search_combined rename column published to published_at;

alter table sent_activity rename column published to published_at;

alter table site rename column published to published_at;
alter table site rename column updated to updated_at;

alter table tag rename column published to published_at;
alter table tag rename column updated to updated_at;

alter table tagline rename column published to published_at;
alter table tagline rename column updated to updated_at;
