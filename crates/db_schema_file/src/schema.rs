// @generated automatically by Diesel CLI.

pub mod sql_types {
  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "actor_type_enum"))]
  pub struct ActorTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "comment_sort_type_enum"))]
  pub struct CommentSortTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "community_follower_state"))]
  pub struct CommunityFollowerState;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "community_notifications_mode_enum"))]
  pub struct CommunityNotificationsModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "community_visibility"))]
  pub struct CommunityVisibility;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "federation_mode_enum"))]
  pub struct FederationModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "listing_type_enum"))]
  pub struct ListingTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "ltree"))]
  pub struct Ltree;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "notification_type_enum"))]
  pub struct NotificationTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_listing_mode_enum"))]
  pub struct PostListingModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_notifications_mode_enum"))]
  pub struct PostNotificationsModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_sort_type_enum"))]
  pub struct PostSortTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "registration_mode_enum"))]
  pub struct RegistrationModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "vote_show_enum"))]
  pub struct VoteShowEnum;
}

diesel::table! {
    admin_add (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        removed -> Bool,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_allow_instance (id) {
        id -> Int4,
        instance_id -> Int4,
        admin_person_id -> Int4,
        allowed -> Bool,
        reason -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_ban (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Bool,
        expires_at -> Nullable<Timestamptz>,
        published_at -> Timestamptz,
        instance_id -> Int4,
    }
}

diesel::table! {
    admin_block_instance (id) {
        id -> Int4,
        instance_id -> Int4,
        admin_person_id -> Int4,
        blocked -> Bool,
        reason -> Nullable<Text>,
        expires_at -> Nullable<Timestamptz>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_comment (id) {
        id -> Int4,
        admin_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_community (id) {
        id -> Int4,
        admin_person_id -> Int4,
        reason -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_person (id) {
        id -> Int4,
        admin_person_id -> Int4,
        reason -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_post (id) {
        id -> Int4,
        admin_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    admin_remove_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    captcha_answer (uuid) {
        uuid -> Uuid,
        answer -> Text,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use diesel_ltree::sql_types::Ltree;

    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        content -> Text,
        removed -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        path -> Ltree,
        distinguished -> Bool,
        language_id -> Int4,
        score -> Int4,
        upvotes -> Int4,
        downvotes -> Int4,
        child_count -> Int4,
        hot_rank -> Float8,
        controversy_rank -> Float8,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        federation_pending -> Bool,
    }
}

diesel::table! {
    comment_actions (person_id, comment_id) {
        person_id -> Int4,
        comment_id -> Int4,
        like_score -> Nullable<Int2>,
        liked_at -> Nullable<Timestamptz>,
        saved_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    comment_report (id) {
        id -> Int4,
        creator_id -> Int4,
        comment_id -> Int4,
        original_comment_text -> Text,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        violates_instance_rules -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityVisibility;

    community (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        sidebar -> Nullable<Text>,
        removed -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        nsfw -> Bool,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamptz,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 255]
        followers_url -> Nullable<Varchar>,
        #[max_length = 255]
        inbox_url -> Varchar,
        posting_restricted_to_mods -> Bool,
        instance_id -> Int4,
        #[max_length = 255]
        moderators_url -> Nullable<Varchar>,
        #[max_length = 255]
        featured_url -> Nullable<Varchar>,
        visibility -> CommunityVisibility,
        #[max_length = 150]
        description -> Nullable<Varchar>,
        random_number -> Int2,
        subscribers -> Int4,
        posts -> Int4,
        comments -> Int4,
        users_active_day -> Int4,
        users_active_week -> Int4,
        users_active_month -> Int4,
        users_active_half_year -> Int4,
        hot_rank -> Float8,
        subscribers_local -> Int4,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        interactions_month -> Int4,
        local_removed -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityFollowerState;
    use super::sql_types::CommunityNotificationsModeEnum;

    community_actions (person_id, community_id) {
        person_id -> Int4,
        community_id -> Int4,
        followed_at -> Nullable<Timestamptz>,
        follow_state -> Nullable<CommunityFollowerState>,
        follow_approver_id -> Nullable<Int4>,
        blocked_at -> Nullable<Timestamptz>,
        became_moderator_at -> Nullable<Timestamptz>,
        received_ban_at -> Nullable<Timestamptz>,
        ban_expires_at -> Nullable<Timestamptz>,
        notifications -> Nullable<CommunityNotificationsModeEnum>,
    }
}

diesel::table! {
    community_community_follow (community_id, target_id) {
        target_id -> Int4,
        community_id -> Int4,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    community_language (community_id, language_id) {
        community_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    community_report (id) {
        id -> Int4,
        creator_id -> Int4,
        community_id -> Int4,
        original_community_name -> Text,
        original_community_title -> Text,
        original_community_description -> Nullable<Text>,
        original_community_sidebar -> Nullable<Text>,
        original_community_icon -> Nullable<Text>,
        original_community_banner -> Nullable<Text>,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    custom_emoji (id) {
        id -> Int4,
        #[max_length = 128]
        shortcode -> Varchar,
        image_url -> Text,
        alt_text -> Text,
        category -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    custom_emoji_keyword (custom_emoji_id, keyword) {
        custom_emoji_id -> Int4,
        #[max_length = 128]
        keyword -> Varchar,
    }
}

diesel::table! {
    email_verification (id) {
        id -> Int4,
        local_user_id -> Int4,
        email -> Text,
        verification_token -> Text,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    federation_allowlist (instance_id) {
        instance_id -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    federation_blocklist (instance_id) {
        instance_id -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    federation_queue_state (instance_id) {
        instance_id -> Int4,
        last_successful_id -> Nullable<Int8>,
        fail_count -> Int4,
        last_retry_at -> Nullable<Timestamptz>,
        last_successful_published_time_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    image_details (link) {
        link -> Text,
        width -> Int4,
        height -> Int4,
        content_type -> Text,
        #[max_length = 50]
        blurhash -> Nullable<Varchar>,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        #[max_length = 255]
        domain -> Varchar,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        software -> Nullable<Varchar>,
        #[max_length = 255]
        version -> Nullable<Varchar>,
    }
}

diesel::table! {
    instance_actions (person_id, instance_id) {
        person_id -> Int4,
        instance_id -> Int4,
        blocked_communities_at -> Nullable<Timestamptz>,
        received_ban_at -> Nullable<Timestamptz>,
        ban_expires_at -> Nullable<Timestamptz>,
        blocked_persons_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    language (id) {
        id -> Int4,
        #[max_length = 3]
        code -> Varchar,
        name -> Text,
    }
}

diesel::table! {
    local_image (pictrs_alias) {
        pictrs_alias -> Text,
        published_at -> Timestamptz,
        person_id -> Nullable<Int4>,
        thumbnail_for_post_id -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ListingTypeEnum;
    use super::sql_types::RegistrationModeEnum;
    use super::sql_types::PostListingModeEnum;
    use super::sql_types::PostSortTypeEnum;
    use super::sql_types::CommentSortTypeEnum;
    use super::sql_types::FederationModeEnum;

    local_site (id) {
        id -> Int4,
        site_id -> Int4,
        site_setup -> Bool,
        community_creation_admin_only -> Bool,
        require_email_verification -> Bool,
        application_question -> Nullable<Text>,
        private_instance -> Bool,
        default_theme -> Text,
        default_post_listing_type -> ListingTypeEnum,
        legal_information -> Nullable<Text>,
        application_email_admins -> Bool,
        slur_filter_regex -> Nullable<Text>,
        actor_name_max_length -> Int4,
        federation_enabled -> Bool,
        captcha_enabled -> Bool,
        #[max_length = 255]
        captcha_difficulty -> Varchar,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        registration_mode -> RegistrationModeEnum,
        reports_email_admins -> Bool,
        federation_signed_fetch -> Bool,
        default_post_listing_mode -> PostListingModeEnum,
        default_post_sort_type -> PostSortTypeEnum,
        default_comment_sort_type -> CommentSortTypeEnum,
        oauth_registration -> Bool,
        post_upvotes -> FederationModeEnum,
        post_downvotes -> FederationModeEnum,
        comment_upvotes -> FederationModeEnum,
        comment_downvotes -> FederationModeEnum,
        default_post_time_range_seconds -> Nullable<Int4>,
        disallow_nsfw_content -> Bool,
        users -> Int4,
        posts -> Int4,
        comments -> Int4,
        communities -> Int4,
        users_active_day -> Int4,
        users_active_week -> Int4,
        users_active_month -> Int4,
        users_active_half_year -> Int4,
        disable_email_notifications -> Bool,
        suggested_communities -> Nullable<Int4>,
        multi_comm_follower -> Int4,
        default_items_per_page -> Int4,
    }
}

diesel::table! {
    local_site_rate_limit (local_site_id) {
        local_site_id -> Int4,
        message_max_requests -> Int4,
        message_interval_seconds -> Int4,
        post_max_requests -> Int4,
        post_interval_seconds -> Int4,
        register_max_requests -> Int4,
        register_interval_seconds -> Int4,
        image_max_requests -> Int4,
        image_interval_seconds -> Int4,
        comment_max_requests -> Int4,
        comment_interval_seconds -> Int4,
        search_max_requests -> Int4,
        search_interval_seconds -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        import_user_settings_max_requests -> Int4,
        import_user_settings_interval_seconds -> Int4,
    }
}

diesel::table! {
    local_site_url_blocklist (id) {
        id -> Int4,
        url -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PostSortTypeEnum;
    use super::sql_types::ListingTypeEnum;
    use super::sql_types::PostListingModeEnum;
    use super::sql_types::CommentSortTypeEnum;
    use super::sql_types::VoteShowEnum;

    local_user (id) {
        id -> Int4,
        person_id -> Int4,
        password_encrypted -> Nullable<Text>,
        email -> Nullable<Text>,
        show_nsfw -> Bool,
        theme -> Text,
        default_post_sort_type -> PostSortTypeEnum,
        default_listing_type -> ListingTypeEnum,
        #[max_length = 20]
        interface_language -> Varchar,
        show_avatars -> Bool,
        send_notifications_to_email -> Bool,
        show_bot_accounts -> Bool,
        show_read_posts -> Bool,
        email_verified -> Bool,
        accepted_application -> Bool,
        totp_2fa_secret -> Nullable<Text>,
        open_links_in_new_tab -> Bool,
        blur_nsfw -> Bool,
        infinite_scroll_enabled -> Bool,
        admin -> Bool,
        post_listing_mode -> PostListingModeEnum,
        totp_2fa_enabled -> Bool,
        enable_keyboard_navigation -> Bool,
        enable_animated_images -> Bool,
        enable_private_messages -> Bool,
        collapse_bot_comments -> Bool,
        default_comment_sort_type -> CommentSortTypeEnum,
        auto_mark_fetched_posts_as_read -> Bool,
        last_donation_notification_at -> Timestamptz,
        hide_media -> Bool,
        default_post_time_range_seconds -> Nullable<Int4>,
        show_score -> Bool,
        show_upvotes -> Bool,
        show_downvotes -> VoteShowEnum,
        show_upvote_percentage -> Bool,
        show_person_votes -> Bool,
        default_items_per_page -> Int4,
    }
}

diesel::table! {
    local_user_keyword_block (local_user_id, keyword) {
        local_user_id -> Int4,
        #[max_length = 50]
        keyword -> Varchar,
    }
}

diesel::table! {
    local_user_language (local_user_id, language_id) {
        local_user_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    login_token (token) {
        token -> Text,
        user_id -> Int4,
        published_at -> Timestamptz,
        ip -> Nullable<Text>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    mod_add_to_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    mod_ban_from_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Bool,
        expires_at -> Nullable<Timestamptz>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityVisibility;

    mod_change_community_visibility (id) {
        id -> Int4,
        community_id -> Int4,
        mod_person_id -> Int4,
        published_at -> Timestamptz,
        visibility -> CommunityVisibility,
    }
}

diesel::table! {
    mod_feature_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        featured -> Bool,
        published_at -> Timestamptz,
        is_featured_community -> Bool,
    }
}

diesel::table! {
    mod_lock_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        locked -> Bool,
        published_at -> Timestamptz,
        reason -> Nullable<Text>,
    }
}

diesel::table! {
    mod_remove_comment (id) {
        id -> Int4,
        mod_person_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    mod_remove_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    mod_transfer_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    modlog_combined (id) {
        id -> Int4,
        published_at -> Timestamptz,
        admin_allow_instance_id -> Nullable<Int4>,
        admin_block_instance_id -> Nullable<Int4>,
        admin_purge_comment_id -> Nullable<Int4>,
        admin_purge_community_id -> Nullable<Int4>,
        admin_purge_person_id -> Nullable<Int4>,
        admin_purge_post_id -> Nullable<Int4>,
        admin_add_id -> Nullable<Int4>,
        mod_add_to_community_id -> Nullable<Int4>,
        admin_ban_id -> Nullable<Int4>,
        mod_ban_from_community_id -> Nullable<Int4>,
        mod_feature_post_id -> Nullable<Int4>,
        mod_lock_post_id -> Nullable<Int4>,
        mod_remove_comment_id -> Nullable<Int4>,
        admin_remove_community_id -> Nullable<Int4>,
        mod_remove_post_id -> Nullable<Int4>,
        mod_transfer_community_id -> Nullable<Int4>,
        mod_change_community_visibility_id -> Nullable<Int4>,
    }
}

diesel::table! {
    multi_community (id) {
        id -> Int4,
        creator_id -> Int4,
        instance_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        title -> Nullable<Varchar>,
        #[max_length = 255]
        description -> Nullable<Varchar>,
        local -> Bool,
        deleted -> Bool,
        ap_id -> Text,
        public_key -> Text,
        private_key -> Nullable<Text>,
        inbox_url -> Text,
        last_refreshed_at -> Timestamptz,
        following_url -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    multi_community_entry (multi_community_id, community_id) {
        multi_community_id -> Int4,
        community_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityFollowerState;

    multi_community_follow (person_id, multi_community_id) {
        multi_community_id -> Int4,
        person_id -> Int4,
        follow_state -> CommunityFollowerState,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NotificationTypeEnum;

    notification (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Nullable<Int4>,
        read -> Bool,
        published_at -> Timestamptz,
        kind -> NotificationTypeEnum,
        post_id -> Nullable<Int4>,
        private_message_id -> Nullable<Int4>,
    }
}

diesel::table! {
    oauth_account (oauth_provider_id, local_user_id) {
        local_user_id -> Int4,
        oauth_provider_id -> Int4,
        oauth_user_id -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    oauth_provider (id) {
        id -> Int4,
        display_name -> Text,
        issuer -> Text,
        authorization_endpoint -> Text,
        token_endpoint -> Text,
        userinfo_endpoint -> Text,
        id_claim -> Text,
        client_id -> Text,
        client_secret -> Text,
        scopes -> Text,
        auto_verify_email -> Bool,
        account_linking_enabled -> Bool,
        enabled -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        use_pkce -> Bool,
    }
}

diesel::table! {
    password_reset_request (id) {
        id -> Int4,
        token -> Text,
        published_at -> Timestamptz,
        local_user_id -> Int4,
    }
}

diesel::table! {
    person (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        display_name -> Nullable<Varchar>,
        avatar -> Nullable<Text>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        ap_id -> Varchar,
        bio -> Nullable<Text>,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamptz,
        banner -> Nullable<Text>,
        deleted -> Bool,
        #[max_length = 255]
        inbox_url -> Varchar,
        matrix_user_id -> Nullable<Text>,
        bot_account -> Bool,
        instance_id -> Int4,
        post_count -> Int4,
        post_score -> Int4,
        comment_count -> Int4,
        comment_score -> Int4,
    }
}

diesel::table! {
    person_actions (person_id, target_id) {
        person_id -> Int4,
        target_id -> Int4,
        followed_at -> Nullable<Timestamptz>,
        follow_pending -> Nullable<Bool>,
        blocked_at -> Nullable<Timestamptz>,
        noted_at -> Nullable<Timestamptz>,
        note -> Nullable<Text>,
        voted_at -> Nullable<Timestamptz>,
        upvotes -> Nullable<Int4>,
        downvotes -> Nullable<Int4>,
    }
}

diesel::table! {
    person_content_combined (id) {
        published_at -> Timestamptz,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        id -> Int4,
    }
}

diesel::table! {
    person_liked_combined (id) {
        liked_at -> Timestamptz,
        like_score -> Int2,
        person_id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        id -> Int4,
    }
}

diesel::table! {
    person_saved_combined (id) {
        saved_at -> Timestamptz,
        person_id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        id -> Int4,
    }
}

diesel::table! {
    post (id) {
        id -> Int4,
        #[max_length = 200]
        name -> Varchar,
        #[max_length = 2000]
        url -> Nullable<Varchar>,
        body -> Nullable<Text>,
        creator_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        locked -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
        nsfw -> Bool,
        embed_title -> Nullable<Text>,
        embed_description -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        embed_video_url -> Nullable<Text>,
        language_id -> Int4,
        featured_community -> Bool,
        featured_local -> Bool,
        url_content_type -> Nullable<Text>,
        alt_text -> Nullable<Text>,
        scheduled_publish_time_at -> Nullable<Timestamptz>,
        comments -> Int4,
        score -> Int4,
        upvotes -> Int4,
        downvotes -> Int4,
        newest_comment_time_necro_at -> Timestamptz,
        newest_comment_time_at -> Timestamptz,
        hot_rank -> Float8,
        hot_rank_active -> Float8,
        controversy_rank -> Float8,
        scaled_rank -> Float8,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        federation_pending -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PostNotificationsModeEnum;

    post_actions (person_id, post_id) {
        person_id -> Int4,
        post_id -> Int4,
        read_at -> Nullable<Timestamptz>,
        read_comments_at -> Nullable<Timestamptz>,
        read_comments_amount -> Nullable<Int4>,
        saved_at -> Nullable<Timestamptz>,
        liked_at -> Nullable<Timestamptz>,
        like_score -> Nullable<Int2>,
        hidden_at -> Nullable<Timestamptz>,
        notifications -> Nullable<PostNotificationsModeEnum>,
    }
}

diesel::table! {
    post_report (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        #[max_length = 200]
        original_post_name -> Varchar,
        original_post_url -> Nullable<Text>,
        original_post_body -> Nullable<Text>,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        violates_instance_rules -> Bool,
    }
}

diesel::table! {
    post_tag (post_id, tag_id) {
        post_id -> Int4,
        tag_id -> Int4,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    private_message (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        content -> Text,
        deleted -> Bool,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        removed -> Bool,
    }
}

diesel::table! {
    private_message_report (id) {
        id -> Int4,
        creator_id -> Int4,
        private_message_id -> Int4,
        original_pm_text -> Text,
        reason -> Text,
        resolved -> Bool,
        resolver_id -> Nullable<Int4>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    received_activity (ap_id) {
        ap_id -> Text,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    registration_application (id) {
        id -> Int4,
        local_user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    remote_image (link) {
        link -> Text,
        published_at -> Timestamptz,
    }
}

diesel::table! {
    report_combined (id) {
        id -> Int4,
        published_at -> Timestamptz,
        post_report_id -> Nullable<Int4>,
        comment_report_id -> Nullable<Int4>,
        private_message_report_id -> Nullable<Int4>,
        community_report_id -> Nullable<Int4>,
    }
}

diesel::table! {
    search_combined (id) {
        published_at -> Timestamptz,
        score -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        community_id -> Nullable<Int4>,
        person_id -> Nullable<Int4>,
        id -> Int4,
        multi_community_id -> Nullable<Int4>,
    }
}

diesel::table! {
    secret (id) {
        id -> Int4,
        jwt_secret -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ActorTypeEnum;

    sent_activity (id) {
        id -> Int8,
        ap_id -> Text,
        data -> Json,
        sensitive -> Bool,
        published_at -> Timestamptz,
        send_inboxes -> Array<Nullable<Text>>,
        send_community_followers_of -> Nullable<Int4>,
        send_all_instances -> Bool,
        actor_type -> ActorTypeEnum,
        actor_apub_id -> Nullable<Text>,
    }
}

diesel::table! {
    site (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
        sidebar -> Nullable<Text>,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 150]
        description -> Nullable<Varchar>,
        #[max_length = 255]
        ap_id -> Varchar,
        last_refreshed_at -> Timestamptz,
        #[max_length = 255]
        inbox_url -> Varchar,
        private_key -> Nullable<Text>,
        public_key -> Text,
        instance_id -> Int4,
        content_warning -> Nullable<Text>,
    }
}

diesel::table! {
    site_language (site_id, language_id) {
        site_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    tag (id) {
        id -> Int4,
        ap_id -> Text,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        display_name -> Nullable<Varchar>,
        description -> Nullable<Text>,
        community_id -> Int4,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        deleted -> Bool,
    }
}

diesel::table! {
    tagline (id) {
        id -> Int4,
        content -> Text,
        published_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(admin_allow_instance -> instance (instance_id));
diesel::joinable!(admin_allow_instance -> person (admin_person_id));
diesel::joinable!(admin_ban -> instance (instance_id));
diesel::joinable!(admin_block_instance -> instance (instance_id));
diesel::joinable!(admin_block_instance -> person (admin_person_id));
diesel::joinable!(admin_purge_comment -> person (admin_person_id));
diesel::joinable!(admin_purge_comment -> post (post_id));
diesel::joinable!(admin_purge_community -> person (admin_person_id));
diesel::joinable!(admin_purge_person -> person (admin_person_id));
diesel::joinable!(admin_purge_post -> community (community_id));
diesel::joinable!(admin_purge_post -> person (admin_person_id));
diesel::joinable!(admin_remove_community -> community (community_id));
diesel::joinable!(admin_remove_community -> person (mod_person_id));
diesel::joinable!(comment -> language (language_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment_actions -> comment (comment_id));
diesel::joinable!(comment_actions -> person (person_id));
diesel::joinable!(comment_report -> comment (comment_id));
diesel::joinable!(community -> instance (instance_id));
diesel::joinable!(community_actions -> community (community_id));
diesel::joinable!(community_language -> community (community_id));
diesel::joinable!(community_language -> language (language_id));
diesel::joinable!(community_report -> community (community_id));
diesel::joinable!(custom_emoji_keyword -> custom_emoji (custom_emoji_id));
diesel::joinable!(email_verification -> local_user (local_user_id));
diesel::joinable!(federation_allowlist -> instance (instance_id));
diesel::joinable!(federation_blocklist -> instance (instance_id));
diesel::joinable!(federation_queue_state -> instance (instance_id));
diesel::joinable!(instance_actions -> instance (instance_id));
diesel::joinable!(instance_actions -> person (person_id));
diesel::joinable!(local_image -> person (person_id));
diesel::joinable!(local_image -> post (thumbnail_for_post_id));
diesel::joinable!(local_site -> multi_community (suggested_communities));
diesel::joinable!(local_site -> person (multi_comm_follower));
diesel::joinable!(local_site -> site (site_id));
diesel::joinable!(local_site_rate_limit -> local_site (local_site_id));
diesel::joinable!(local_user -> person (person_id));
diesel::joinable!(local_user_keyword_block -> local_user (local_user_id));
diesel::joinable!(local_user_language -> language (language_id));
diesel::joinable!(local_user_language -> local_user (local_user_id));
diesel::joinable!(login_token -> local_user (user_id));
diesel::joinable!(mod_add_to_community -> community (community_id));
diesel::joinable!(mod_ban_from_community -> community (community_id));
diesel::joinable!(mod_change_community_visibility -> community (community_id));
diesel::joinable!(mod_change_community_visibility -> person (mod_person_id));
diesel::joinable!(mod_feature_post -> person (mod_person_id));
diesel::joinable!(mod_feature_post -> post (post_id));
diesel::joinable!(mod_lock_post -> person (mod_person_id));
diesel::joinable!(mod_lock_post -> post (post_id));
diesel::joinable!(mod_remove_comment -> comment (comment_id));
diesel::joinable!(mod_remove_comment -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> post (post_id));
diesel::joinable!(mod_transfer_community -> community (community_id));
diesel::joinable!(modlog_combined -> admin_add (admin_add_id));
diesel::joinable!(modlog_combined -> admin_allow_instance (admin_allow_instance_id));
diesel::joinable!(modlog_combined -> admin_ban (admin_ban_id));
diesel::joinable!(modlog_combined -> admin_block_instance (admin_block_instance_id));
diesel::joinable!(modlog_combined -> admin_purge_comment (admin_purge_comment_id));
diesel::joinable!(modlog_combined -> admin_purge_community (admin_purge_community_id));
diesel::joinable!(modlog_combined -> admin_purge_person (admin_purge_person_id));
diesel::joinable!(modlog_combined -> admin_purge_post (admin_purge_post_id));
diesel::joinable!(modlog_combined -> admin_remove_community (admin_remove_community_id));
diesel::joinable!(modlog_combined -> mod_add_to_community (mod_add_to_community_id));
diesel::joinable!(modlog_combined -> mod_ban_from_community (mod_ban_from_community_id));
diesel::joinable!(modlog_combined -> mod_change_community_visibility (mod_change_community_visibility_id));
diesel::joinable!(modlog_combined -> mod_feature_post (mod_feature_post_id));
diesel::joinable!(modlog_combined -> mod_lock_post (mod_lock_post_id));
diesel::joinable!(modlog_combined -> mod_remove_comment (mod_remove_comment_id));
diesel::joinable!(modlog_combined -> mod_remove_post (mod_remove_post_id));
diesel::joinable!(modlog_combined -> mod_transfer_community (mod_transfer_community_id));
diesel::joinable!(multi_community -> instance (instance_id));
diesel::joinable!(multi_community -> person (creator_id));
diesel::joinable!(multi_community_entry -> community (community_id));
diesel::joinable!(multi_community_entry -> multi_community (multi_community_id));
diesel::joinable!(multi_community_follow -> multi_community (multi_community_id));
diesel::joinable!(multi_community_follow -> person (person_id));
diesel::joinable!(notification -> comment (comment_id));
diesel::joinable!(notification -> person (recipient_id));
diesel::joinable!(notification -> post (post_id));
diesel::joinable!(notification -> private_message (private_message_id));
diesel::joinable!(oauth_account -> local_user (local_user_id));
diesel::joinable!(oauth_account -> oauth_provider (oauth_provider_id));
diesel::joinable!(password_reset_request -> local_user (local_user_id));
diesel::joinable!(person -> instance (instance_id));
diesel::joinable!(person_content_combined -> comment (comment_id));
diesel::joinable!(person_content_combined -> post (post_id));
diesel::joinable!(person_liked_combined -> comment (comment_id));
diesel::joinable!(person_liked_combined -> person (person_id));
diesel::joinable!(person_liked_combined -> post (post_id));
diesel::joinable!(person_saved_combined -> comment (comment_id));
diesel::joinable!(person_saved_combined -> person (person_id));
diesel::joinable!(person_saved_combined -> post (post_id));
diesel::joinable!(post -> community (community_id));
diesel::joinable!(post -> language (language_id));
diesel::joinable!(post -> person (creator_id));
diesel::joinable!(post_actions -> person (person_id));
diesel::joinable!(post_actions -> post (post_id));
diesel::joinable!(post_report -> post (post_id));
diesel::joinable!(post_tag -> post (post_id));
diesel::joinable!(post_tag -> tag (tag_id));
diesel::joinable!(private_message_report -> private_message (private_message_id));
diesel::joinable!(registration_application -> local_user (local_user_id));
diesel::joinable!(registration_application -> person (admin_id));
diesel::joinable!(report_combined -> comment_report (comment_report_id));
diesel::joinable!(report_combined -> community_report (community_report_id));
diesel::joinable!(report_combined -> post_report (post_report_id));
diesel::joinable!(report_combined -> private_message_report (private_message_report_id));
diesel::joinable!(search_combined -> comment (comment_id));
diesel::joinable!(search_combined -> community (community_id));
diesel::joinable!(search_combined -> multi_community (multi_community_id));
diesel::joinable!(search_combined -> person (person_id));
diesel::joinable!(search_combined -> post (post_id));
diesel::joinable!(site -> instance (instance_id));
diesel::joinable!(site_language -> language (language_id));
diesel::joinable!(site_language -> site (site_id));
diesel::joinable!(tag -> community (community_id));

diesel::allow_tables_to_appear_in_same_query!(
  admin_add,
  admin_allow_instance,
  admin_ban,
  admin_block_instance,
  admin_purge_comment,
  admin_purge_community,
  admin_purge_person,
  admin_purge_post,
  admin_remove_community,
  captcha_answer,
  comment,
  comment_actions,
  comment_report,
  community,
  community_actions,
  community_community_follow,
  community_language,
  community_report,
  custom_emoji,
  custom_emoji_keyword,
  email_verification,
  federation_allowlist,
  federation_blocklist,
  federation_queue_state,
  image_details,
  instance,
  instance_actions,
  language,
  local_image,
  local_site,
  local_site_rate_limit,
  local_site_url_blocklist,
  local_user,
  local_user_keyword_block,
  local_user_language,
  login_token,
  mod_add_to_community,
  mod_ban_from_community,
  mod_change_community_visibility,
  mod_feature_post,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_post,
  mod_transfer_community,
  modlog_combined,
  multi_community,
  multi_community_entry,
  multi_community_follow,
  notification,
  oauth_account,
  oauth_provider,
  password_reset_request,
  person,
  person_actions,
  person_content_combined,
  person_liked_combined,
  person_saved_combined,
  post,
  post_actions,
  post_report,
  post_tag,
  private_message,
  private_message_report,
  received_activity,
  registration_application,
  remote_image,
  report_combined,
  search_combined,
  secret,
  sent_activity,
  site,
  site_language,
  tag,
  tagline,
);
