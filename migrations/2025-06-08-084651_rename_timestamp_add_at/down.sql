alter table admin_allow_instance rename published_at to published;

alter table admin_block_instance rename column expires_at to expires;
alter table admin_block_instance rename column published_at to published;

alter table admin_purge_comment rename column published_at to published;
alter table admin_purge_community rename column published_at to published;
alter table admin_purge_person rename column published_at to published;
alter table admin_purge_post rename column published_at to published;
alter table captcha_answer rename column published_at to published;

alter table comment rename column published_at to published;
alter table comment rename column updated_at to updated;
alter table comment_actions rename column liked_at to liked;
alter table comment_actions rename column saved_at to saved;
alter table comment_reply rename column published_at to published;
alter table comment_report rename column published_at to published;
alter table comment_report rename column updated_at to updated;
alter table community rename column published_at to published;
alter table community rename column updated_at to updated;

alter table community_actions rename column followed_at to followed;
alter table community_actions rename column blocked_at to blocked;
alter table community_actions rename column became_moderator_at to became_moderator;
alter table community_actions rename column received_ban_at to received_ban;
alter table community_actions rename column ban_expires_at to ban_expires;


alter table community_report rename column published_at to published;
alter table community_report rename column updated_at to updated;

alter table custom_emoji rename column published_at to published;
alter table custom_emoji rename column updated_at to updated;
alter table email_verification rename column published_at to published;


alter table federation_allowlist rename column published_at to published;
alter table federation_allowlist rename column updated_at to updated;

alter table federation_blocklist rename column published_at to published;
alter table federation_blocklist rename column updated_at to updated;
alter table federation_blocklist rename column expires_at to expires;

alter table federation_queue_state rename column last_retry_at to last_retry;
alter table federation_queue_state rename column last_successful_published_time_at to last_successful_published_time;

alter table inbox_combined rename column published_at to published;

alter table instance rename column published_at to published;
alter table instance rename column updated_at to updated;

alter table instance_actions rename column blocked_at to blocked;
alter table instance_actions rename column received_ban_at to received_ban;
alter table instance_actions rename column ban_expires_at to ban_expires;
alter table local_image rename column published_at to published;

alter table local_site rename column published_at to published;
alter table local_site rename column updated_at to updated;

alter table local_site_rate_limit rename column published_at to published;
alter table local_site_rate_limit rename column updated_at to updated;

alter table local_site_url_blocklist rename column published_at to published;
alter table local_site_url_blocklist rename column updated_at to updated;

alter table local_user rename column last_donation_notification_at to last_donation_notification;

alter table login_token rename column published_at to published;

alter table mod_add rename column published_at to published;
alter table mod_add_community rename column published_at to published;

alter table mod_ban rename column published_at to published;
alter table mod_ban rename column expires_at to expires;

alter table mod_ban_from_community rename column published_at to published;
alter table mod_ban_from_community rename column expires_at to expires;

alter table mod_change_community_visibility rename column published_at to published;
alter table mod_feature_post rename column published_at to published;
alter table mod_lock_post rename column published_at to published;

alter table mod_remove_comment rename column published_at to published;
alter table mod_remove_community rename column published_at to published;
alter table mod_remove_post rename column published_at to published;
alter table mod_transfer_community rename column published_at to published;

alter table modlog_combined rename column published_at to published;

alter table oauth_account rename column published_at to published;
alter table oauth_account rename column updated_at to updated;

alter table oauth_provider rename column published_at to published;
alter table oauth_provider rename column updated_at to updated;

alter table password_reset_request rename column published_at to published;

alter table person rename column published_at to published;
alter table person rename column updated_at to updated;

alter table person_actions rename column followed_at to followed;
alter table person_actions rename column blocked_at to blocked;

alter table person_ban rename column published_at to published;

alter table person_comment_mention rename column published_at to published;

alter table person_content_combined rename column published_at to published;

alter table person_liked_combined rename column liked_at to liked;

alter table person_post_mention rename column published_at to published;

alter table person_saved_combined rename column saved_at to saved;

alter table post rename column published_at to published;
alter table post rename column updated_at to updated;
alter table post rename column scheduled_publish_time_at to scheduled_publish_time;
alter table post rename column newest_comment_time_at to newest_comment_time;
alter table post rename column newest_comment_time_necro_at to newest_comment_time_necro;

alter table post_actions rename column read_at to read;
alter table post_actions rename column read_comments_at to read_comments;
alter table post_actions rename column saved_at to saved;
alter table post_actions rename column liked_at to liked;
alter table post_actions rename column hidden_at to hidden;

alter table post_report rename column published_at to published;
alter table post_report rename column updated_at to updated;

alter table post_tag rename column published_at to published;

alter table private_message rename column published_at to published;
alter table private_message rename column updated_at to updated;

alter table private_message_report rename column published_at to published;
alter table private_message_report rename column updated_at to updated;

alter table received_activity rename column published_at to published;

alter table registration_application rename column published_at to published;
alter table remote_image rename column published_at to published;

alter table report_combined rename column published_at to published;
alter table search_combined rename column published_at to published;

alter table sent_activity rename column published_at to published;

alter table site rename column published_at to published;
alter table site rename column updated_at to updated;

alter table tag rename column published_at to published;
alter table tag rename column updated_at to updated;

alter table tagline rename column published_at to published;
alter table tagline rename column updated_at to updated;
