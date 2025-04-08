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
  #[diesel(postgres_type(name = "post_listing_mode_enum"))]
  pub struct PostListingModeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "post_sort_type_enum"))]
  pub struct PostSortTypeEnum;

  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "registration_mode_enum"))]
  pub struct RegistrationModeEnum;
}

diesel::table! {
    admin_allow_instance (id) {
        id -> Int4,
        instance_id -> Int4,
        admin_person_id -> Int4,
        allowed -> Bool,
        reason -> Nullable<Text>,
        published -> Timestamptz,
    }
}

diesel::table! {
    admin_block_instance (id) {
        id -> Int4,
        instance_id -> Int4,
        admin_person_id -> Int4,
        blocked -> Bool,
        reason -> Nullable<Text>,
        expires -> Nullable<Timestamptz>,
        published -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_comment (id) {
        id -> Int4,
        admin_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        published -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_community (id) {
        id -> Int4,
        admin_person_id -> Int4,
        reason -> Nullable<Text>,
        published -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_person (id) {
        id -> Int4,
        admin_person_id -> Int4,
        reason -> Nullable<Text>,
        published -> Timestamptz,
    }
}

diesel::table! {
    admin_purge_post (id) {
        id -> Int4,
        admin_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        published -> Timestamptz,
    }
}

diesel::table! {
    captcha_answer (uuid) {
        uuid -> Uuid,
        answer -> Text,
        published -> Timestamptz,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
        deleted -> Bool,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        path -> Ltree,
        distinguished -> Bool,
        language_id -> Int4,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
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
        liked -> Nullable<Timestamptz>,
        saved -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    comment_reply (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamptz,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        subscribers -> Int8,
        posts -> Int8,
        comments -> Int8,
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
        hot_rank -> Float8,
        subscribers_local -> Int8,
        report_count -> Int2,
        unresolved_report_count -> Int2,
        interactions_month -> Int8,
        local_removed -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityFollowerState;

    community_actions (person_id, community_id) {
        community_id -> Int4,
        person_id -> Int4,
        followed -> Nullable<Timestamptz>,
        follow_state -> Nullable<CommunityFollowerState>,
        follow_approver_id -> Nullable<Int4>,
        blocked -> Nullable<Timestamptz>,
        became_moderator -> Nullable<Timestamptz>,
        received_ban -> Nullable<Timestamptz>,
        ban_expires -> Nullable<Timestamptz>,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        published -> Timestamptz,
    }
}

diesel::table! {
    federation_allowlist (instance_id) {
        instance_id -> Int4,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    federation_blocklist (instance_id) {
        instance_id -> Int4,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
        expires -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    federation_queue_state (instance_id) {
        instance_id -> Int4,
        last_successful_id -> Nullable<Int8>,
        fail_count -> Int4,
        last_retry -> Nullable<Timestamptz>,
        last_successful_published_time -> Nullable<Timestamptz>,
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
    inbox_combined (id) {
        id -> Int4,
        published -> Timestamptz,
        comment_reply_id -> Nullable<Int4>,
        person_comment_mention_id -> Nullable<Int4>,
        person_post_mention_id -> Nullable<Int4>,
        private_message_id -> Nullable<Int4>,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        #[max_length = 255]
        domain -> Varchar,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        blocked -> Nullable<Timestamptz>,
        received_ban -> Nullable<Timestamptz>,
        ban_expires -> Nullable<Timestamptz>,
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
        local_user_id -> Nullable<Int4>,
        pictrs_alias -> Text,
        published -> Timestamptz,
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
        hide_modlog_mod_names -> Bool,
        application_email_admins -> Bool,
        slur_filter_regex -> Nullable<Text>,
        actor_name_max_length -> Int4,
        federation_enabled -> Bool,
        captcha_enabled -> Bool,
        #[max_length = 255]
        captcha_difficulty -> Varchar,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        users -> Int8,
        posts -> Int8,
        comments -> Int8,
        communities -> Int8,
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
    }
}

diesel::table! {
    local_site_rate_limit (local_site_id) {
        local_site_id -> Int4,
        message -> Int4,
        message_per_second -> Int4,
        post -> Int4,
        post_per_second -> Int4,
        register -> Int4,
        register_per_second -> Int4,
        image -> Int4,
        image_per_second -> Int4,
        comment -> Int4,
        comment_per_second -> Int4,
        search -> Int4,
        search_per_second -> Int4,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
        import_user_settings -> Int4,
        import_user_settings_per_second -> Int4,
    }
}

diesel::table! {
    local_site_url_blocklist (id) {
        id -> Int4,
        url -> Text,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PostSortTypeEnum;
    use super::sql_types::ListingTypeEnum;
    use super::sql_types::PostListingModeEnum;
    use super::sql_types::CommentSortTypeEnum;

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
        last_donation_notification -> Timestamptz,
        hide_media -> Bool,
        default_post_time_range_seconds -> Nullable<Int4>,
        show_score -> Bool,
        show_upvotes -> Bool,
        show_downvotes -> Bool,
        show_upvote_percentage -> Bool,
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
        published -> Timestamptz,
        ip -> Nullable<Text>,
        user_agent -> Nullable<Text>,
    }
}

diesel::table! {
    mod_add (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        removed -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    mod_add_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    mod_ban (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Bool,
        expires -> Nullable<Timestamptz>,
        published -> Timestamptz,
        instance_id -> Int4,
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
        expires -> Nullable<Timestamptz>,
        published -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::CommunityVisibility;

    mod_change_community_visibility (id) {
        id -> Int4,
        community_id -> Int4,
        mod_person_id -> Int4,
        published -> Timestamptz,
        reason -> Nullable<Text>,
        visibility -> CommunityVisibility,
    }
}

diesel::table! {
    mod_feature_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        featured -> Bool,
        published -> Timestamptz,
        is_featured_community -> Bool,
    }
}

diesel::table! {
    mod_lock_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        locked -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    mod_remove_comment (id) {
        id -> Int4,
        mod_person_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    mod_remove_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    mod_remove_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    mod_transfer_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        published -> Timestamptz,
    }
}

diesel::table! {
    modlog_combined (id) {
        id -> Int4,
        published -> Timestamptz,
        admin_allow_instance_id -> Nullable<Int4>,
        admin_block_instance_id -> Nullable<Int4>,
        admin_purge_comment_id -> Nullable<Int4>,
        admin_purge_community_id -> Nullable<Int4>,
        admin_purge_person_id -> Nullable<Int4>,
        admin_purge_post_id -> Nullable<Int4>,
        mod_add_id -> Nullable<Int4>,
        mod_add_community_id -> Nullable<Int4>,
        mod_ban_id -> Nullable<Int4>,
        mod_ban_from_community_id -> Nullable<Int4>,
        mod_feature_post_id -> Nullable<Int4>,
        mod_lock_post_id -> Nullable<Int4>,
        mod_remove_comment_id -> Nullable<Int4>,
        mod_remove_community_id -> Nullable<Int4>,
        mod_remove_post_id -> Nullable<Int4>,
        mod_transfer_community_id -> Nullable<Int4>,
        mod_change_community_visibility_id -> Nullable<Int4>,
    }
}

diesel::table! {
    oauth_account (oauth_provider_id, local_user_id) {
        local_user_id -> Int4,
        oauth_provider_id -> Int4,
        oauth_user_id -> Text,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
        use_pkce -> Bool,
    }
}

diesel::table! {
    password_reset_request (id) {
        id -> Int4,
        token -> Text,
        published -> Timestamptz,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        post_count -> Int8,
        post_score -> Int8,
        comment_count -> Int8,
        comment_score -> Int8,
    }
}

diesel::table! {
    person_actions (person_id, target_id) {
        target_id -> Int4,
        person_id -> Int4,
        followed -> Nullable<Timestamptz>,
        follow_pending -> Nullable<Bool>,
        blocked -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    person_ban (person_id) {
        person_id -> Int4,
        published -> Timestamptz,
    }
}

diesel::table! {
    person_comment_mention (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    person_content_combined (id) {
        id -> Int4,
        published -> Timestamptz,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
    }
}

diesel::table! {
    person_post_mention (id) {
        id -> Int4,
        recipient_id -> Int4,
        post_id -> Int4,
        read -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    person_saved_combined (id) {
        id -> Int4,
        saved -> Timestamptz,
        person_id -> Int4,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        scheduled_publish_time -> Nullable<Timestamptz>,
        comments -> Int8,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        newest_comment_time_necro -> Timestamptz,
        newest_comment_time -> Timestamptz,
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
    post_actions (person_id, post_id) {
        post_id -> Int4,
        person_id -> Int4,
        read -> Nullable<Timestamptz>,
        read_comments -> Nullable<Timestamptz>,
        read_comments_amount -> Nullable<Int8>,
        saved -> Nullable<Timestamptz>,
        liked -> Nullable<Timestamptz>,
        like_score -> Nullable<Int2>,
        hidden -> Nullable<Timestamptz>,
        subscribed -> Nullable<Bool>
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
        violates_instance_rules -> Bool,
    }
}

diesel::table! {
    post_tag (post_id, tag_id) {
        post_id -> Int4,
        tag_id -> Int4,
        published -> Timestamptz,
    }
}

diesel::table! {
    previously_run_sql (id) {
        id -> Bool,
        content -> Text,
    }
}

diesel::table! {
    private_message (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        content -> Text,
        deleted -> Bool,
        read -> Bool,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    received_activity (ap_id) {
        ap_id -> Text,
        published -> Timestamptz,
    }
}

diesel::table! {
    registration_application (id) {
        id -> Int4,
        local_user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published -> Timestamptz,
    }
}

diesel::table! {
    remote_image (link) {
        link -> Text,
        published -> Timestamptz,
    }
}

diesel::table! {
    report_combined (id) {
        id -> Int4,
        published -> Timestamptz,
        post_report_id -> Nullable<Int4>,
        comment_report_id -> Nullable<Int4>,
        private_message_report_id -> Nullable<Int4>,
        community_report_id -> Nullable<Int4>,
    }
}

diesel::table! {
    search_combined (id) {
        id -> Int4,
        published -> Timestamptz,
        score -> Int8,
        post_id -> Nullable<Int4>,
        comment_id -> Nullable<Int4>,
        community_id -> Nullable<Int4>,
        person_id -> Nullable<Int4>,
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
        published -> Timestamptz,
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
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
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
        display_name -> Text,
        community_id -> Int4,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
        deleted -> Bool,
    }
}

diesel::table! {
    tagline (id) {
        id -> Int4,
        content -> Text,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
    }
}

diesel::joinable!(admin_allow_instance -> instance (instance_id));
diesel::joinable!(admin_allow_instance -> person (admin_person_id));
diesel::joinable!(admin_block_instance -> instance (instance_id));
diesel::joinable!(admin_block_instance -> person (admin_person_id));
diesel::joinable!(admin_purge_comment -> person (admin_person_id));
diesel::joinable!(admin_purge_comment -> post (post_id));
diesel::joinable!(admin_purge_community -> person (admin_person_id));
diesel::joinable!(admin_purge_person -> person (admin_person_id));
diesel::joinable!(admin_purge_post -> community (community_id));
diesel::joinable!(admin_purge_post -> person (admin_person_id));
diesel::joinable!(comment -> language (language_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment_actions -> comment (comment_id));
diesel::joinable!(comment_actions -> person (person_id));
diesel::joinable!(comment_reply -> comment (comment_id));
diesel::joinable!(comment_reply -> person (recipient_id));
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
diesel::joinable!(inbox_combined -> comment_reply (comment_reply_id));
diesel::joinable!(inbox_combined -> person_comment_mention (person_comment_mention_id));
diesel::joinable!(inbox_combined -> person_post_mention (person_post_mention_id));
diesel::joinable!(inbox_combined -> private_message (private_message_id));
diesel::joinable!(instance_actions -> instance (instance_id));
diesel::joinable!(instance_actions -> person (person_id));
diesel::joinable!(local_image -> local_user (local_user_id));
diesel::joinable!(local_site -> site (site_id));
diesel::joinable!(local_site_rate_limit -> local_site (local_site_id));
diesel::joinable!(local_user -> person (person_id));
diesel::joinable!(local_user_keyword_block -> local_user (local_user_id));
diesel::joinable!(local_user_language -> language (language_id));
diesel::joinable!(local_user_language -> local_user (local_user_id));
diesel::joinable!(login_token -> local_user (user_id));
diesel::joinable!(mod_add_community -> community (community_id));
diesel::joinable!(mod_ban_from_community -> community (community_id));
diesel::joinable!(mod_change_community_visibility -> community (community_id));
diesel::joinable!(mod_change_community_visibility -> person (mod_person_id));
diesel::joinable!(mod_feature_post -> person (mod_person_id));
diesel::joinable!(mod_feature_post -> post (post_id));
diesel::joinable!(mod_lock_post -> person (mod_person_id));
diesel::joinable!(mod_lock_post -> post (post_id));
diesel::joinable!(mod_remove_comment -> comment (comment_id));
diesel::joinable!(mod_remove_comment -> person (mod_person_id));
diesel::joinable!(mod_remove_community -> community (community_id));
diesel::joinable!(mod_remove_community -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> post (post_id));
diesel::joinable!(mod_transfer_community -> community (community_id));
diesel::joinable!(modlog_combined -> admin_allow_instance (admin_allow_instance_id));
diesel::joinable!(modlog_combined -> admin_block_instance (admin_block_instance_id));
diesel::joinable!(modlog_combined -> admin_purge_comment (admin_purge_comment_id));
diesel::joinable!(modlog_combined -> admin_purge_community (admin_purge_community_id));
diesel::joinable!(modlog_combined -> admin_purge_person (admin_purge_person_id));
diesel::joinable!(modlog_combined -> admin_purge_post (admin_purge_post_id));
diesel::joinable!(modlog_combined -> mod_add (mod_add_id));
diesel::joinable!(modlog_combined -> mod_add_community (mod_add_community_id));
diesel::joinable!(modlog_combined -> mod_ban (mod_ban_id));
diesel::joinable!(modlog_combined -> mod_ban_from_community (mod_ban_from_community_id));
diesel::joinable!(modlog_combined -> mod_change_community_visibility (mod_change_community_visibility_id));
diesel::joinable!(modlog_combined -> mod_feature_post (mod_feature_post_id));
diesel::joinable!(modlog_combined -> mod_lock_post (mod_lock_post_id));
diesel::joinable!(modlog_combined -> mod_remove_comment (mod_remove_comment_id));
diesel::joinable!(modlog_combined -> mod_remove_community (mod_remove_community_id));
diesel::joinable!(modlog_combined -> mod_remove_post (mod_remove_post_id));
diesel::joinable!(modlog_combined -> mod_transfer_community (mod_transfer_community_id));
diesel::joinable!(oauth_account -> local_user (local_user_id));
diesel::joinable!(oauth_account -> oauth_provider (oauth_provider_id));
diesel::joinable!(password_reset_request -> local_user (local_user_id));
diesel::joinable!(person -> instance (instance_id));
diesel::joinable!(person_ban -> person (person_id));
diesel::joinable!(person_comment_mention -> comment (comment_id));
diesel::joinable!(person_comment_mention -> person (recipient_id));
diesel::joinable!(person_content_combined -> comment (comment_id));
diesel::joinable!(person_content_combined -> post (post_id));
diesel::joinable!(person_post_mention -> person (recipient_id));
diesel::joinable!(person_post_mention -> post (post_id));
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
diesel::joinable!(search_combined -> person (person_id));
diesel::joinable!(search_combined -> post (post_id));
diesel::joinable!(site -> instance (instance_id));
diesel::joinable!(site_language -> language (language_id));
diesel::joinable!(site_language -> site (site_id));
diesel::joinable!(tag -> community (community_id));

diesel::allow_tables_to_appear_in_same_query!(
  admin_allow_instance,
  admin_block_instance,
  admin_purge_comment,
  admin_purge_community,
  admin_purge_person,
  admin_purge_post,
  captcha_answer,
  comment,
  comment_actions,
  comment_reply,
  comment_report,
  community,
  community_actions,
  community_language,
  community_report,
  custom_emoji,
  custom_emoji_keyword,
  email_verification,
  federation_allowlist,
  federation_blocklist,
  federation_queue_state,
  image_details,
  inbox_combined,
  instance,
  instance_actions,
  language,
  local_image,
  local_site,
  local_site_rate_limit,
  local_site_url_blocklist,
  local_user,
  local_user_language,
  local_user_keyword_block,
  login_token,
  mod_add,
  mod_add_community,
  mod_ban,
  mod_ban_from_community,
  mod_change_community_visibility,
  mod_feature_post,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_community,
  mod_remove_post,
  mod_transfer_community,
  modlog_combined,
  oauth_account,
  oauth_provider,
  password_reset_request,
  person,
  person_actions,
  person_ban,
  person_comment_mention,
  person_content_combined,
  person_post_mention,
  person_saved_combined,
  post,
  post_actions,
  post_report,
  post_tag,
  previously_run_sql,
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
