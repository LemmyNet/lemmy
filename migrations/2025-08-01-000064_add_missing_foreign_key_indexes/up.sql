CREATE INDEX idx_registration_application_admin ON registration_application (admin_id);

CREATE INDEX idx_admin_allow_instance_admin ON admin_allow_instance (admin_person_id);

CREATE INDEX idx_admin_block_instance_admin ON admin_block_instance (admin_person_id);

CREATE INDEX idx_admin_purge_comment_admin ON admin_purge_comment (admin_person_id);

CREATE INDEX idx_admin_purge_community_admin ON admin_purge_community (admin_person_id);

CREATE INDEX idx_admin_purge_person_admin ON admin_purge_person (admin_person_id);

CREATE INDEX idx_admin_purge_post_admin ON admin_purge_post (admin_person_id);

CREATE INDEX idx_mod_remove_comment_comment ON mod_remove_comment (comment_id);

CREATE INDEX idx_person_liked_combined_comment ON person_liked_combined (comment_id)
WHERE
    comment_id IS NOT NULL;

CREATE INDEX idx_person_saved_combined_comment ON person_saved_combined (comment_id)
WHERE
    comment_id IS NOT NULL;

CREATE INDEX idx_comment_report_creator ON comment_report (creator_id);

CREATE INDEX idx_community_report_creator ON community_report (creator_id);

CREATE INDEX idx_post_report_creator ON post_report (creator_id);

CREATE INDEX idx_private_message_creator ON private_message (creator_id);

CREATE INDEX idx_private_message_report_creator ON private_message_report (creator_id);

CREATE INDEX idx_admin_purge_post_community ON admin_purge_post (community_id);

CREATE INDEX idx_mod_add_community_community ON mod_add_community (community_id);

CREATE INDEX idx_mod_ban_from_community_community ON mod_ban_from_community (community_id);

CREATE INDEX idx_mod_change_community_visibility_community ON mod_change_community_visibility (community_id);

CREATE INDEX idx_mod_remove_community_community ON mod_remove_community (community_id);

CREATE INDEX idx_mod_transfer_community_community ON mod_transfer_community (community_id);

CREATE INDEX idx_tag_community ON tag (community_id);

CREATE INDEX idx_community_actions_follow_approver ON community_actions (follow_approver_id);

CREATE INDEX idx_admin_allow_instance_instance ON admin_allow_instance (instance_id);

CREATE INDEX idx_admin_block_instance_instance ON admin_block_instance (instance_id);

CREATE INDEX idx_community_instance ON community (instance_id);

CREATE INDEX idx_mod_ban_instance ON mod_ban (instance_id);

CREATE INDEX idx_multi_community_instance ON multi_community (instance_id);

CREATE INDEX idx_person_instance ON person (instance_id);

CREATE INDEX idx_community_language_language ON community_language (language_id);

CREATE INDEX idx_local_user_language_language ON local_user_language (language_id);

CREATE INDEX idx_site_language_language ON site_language (language_id);

CREATE INDEX idx_email_verification_user ON email_verification (local_user_id);

CREATE INDEX idx_oauth_account_user ON oauth_account (local_user_id);

CREATE INDEX idx_password_reset_request_user ON password_reset_request (local_user_id);

CREATE INDEX idx_modlog_combined_mod_change_community_visibility_id ON modlog_combined (mod_change_community_visibility_id)
WHERE
    mod_change_community_visibility_id IS NOT NULL;

CREATE INDEX idx_mod_add_community_mod ON mod_add_community (mod_person_id);

CREATE INDEX idx_mod_add_mod ON mod_add (mod_person_id);

CREATE INDEX idx_mod_ban_from_community_mod ON mod_ban_from_community (mod_person_id);

CREATE INDEX idx_mod_ban_mod ON mod_ban (mod_person_id);

CREATE INDEX idx_mod_change_community_visibility_mod ON mod_change_community_visibility (mod_person_id);

CREATE INDEX idx_mod_feature_post_mod ON mod_feature_post (mod_person_id);

CREATE INDEX idx_mod_lock_post_mod ON mod_lock_post (mod_person_id);

CREATE INDEX idx_mod_remove_comment_mod ON mod_remove_comment (mod_person_id);

CREATE INDEX idx_mod_remove_community_mod ON mod_remove_community (mod_person_id);

CREATE INDEX idx_mod_remove_post_mod ON mod_remove_post (mod_person_id);

CREATE INDEX idx_mod_transfer_community_mod ON mod_transfer_community (mod_person_id);

CREATE INDEX idx_local_site_system_account ON local_site (system_account);

CREATE INDEX idx_search_combined_multi_community ON search_combined (multi_community_id)
WHERE
    multi_community_id IS NOT NULL;

CREATE INDEX idx_mod_add_community_other_person ON mod_add_community (other_person_id);

CREATE INDEX idx_mod_add_other_person ON mod_add (other_person_id);

CREATE INDEX idx_mod_ban_from_community_other_person ON mod_ban_from_community (other_person_id);

CREATE INDEX idx_mod_other_person ON mod_ban (other_person_id);

CREATE INDEX idx_mod_transfer_community_other_person ON mod_transfer_community (other_person_id);

CREATE INDEX idx_admin_purge_comment_post ON admin_purge_comment (post_id);

CREATE INDEX idx_mod_feature_post_post ON mod_feature_post (post_id);

CREATE INDEX idx_mod_lock_post_post ON mod_lock_post (post_id);

CREATE INDEX idx_mod_remove_post_post ON mod_remove_post (post_id);

CREATE INDEX idx_person_liked_combined_post ON person_liked_combined (post_id)
WHERE
    post_id IS NOT NULL;

CREATE INDEX idx_person_saved_combined_post ON person_saved_combined (post_id)
WHERE
    post_id IS NOT NULL;

CREATE INDEX idx_private_message_recipient ON private_message (recipient_id);

CREATE INDEX idx_comment_report_resolver ON comment_report (resolver_id);

CREATE INDEX idx_community_report_resolver ON community_report (resolver_id);

CREATE INDEX idx_post_report_resolver ON post_report (resolver_id);

CREATE INDEX idx_private_message_report_resolver ON private_message_report (resolver_id);

CREATE INDEX idx_local_site_suggested_communities ON local_site (suggested_communities);

CREATE INDEX idx_post_tag_tag ON post_tag (tag_id);

CREATE INDEX idx_local_image_thumbnail_post ON local_image (thumbnail_for_post_id);

