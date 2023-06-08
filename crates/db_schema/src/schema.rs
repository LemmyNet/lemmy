// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "listing_type_enum"))]
    pub struct ListingTypeEnum;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "registration_mode_enum"))]
    pub struct RegistrationModeEnum;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "sort_type_enum"))]
    pub struct SortTypeEnum;
}

diesel::table! {
    activity (id) {
        id -> Int4,
        data -> Jsonb,
        local -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        ap_id -> Text,
        sensitive -> Bool,
    }
}

diesel::table! {
    admin_purge_comment (id) {
        id -> Int4,
        admin_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    admin_purge_community (id) {
        id -> Int4,
        admin_person_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    admin_purge_person (id) {
        id -> Int4,
        admin_person_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    admin_purge_post (id) {
        id -> Int4,
        admin_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::{Bool, Int4, Nullable, Text, Timestamp, Varchar};
    use diesel_ltree::sql_types::Ltree;

    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        content -> Text,
        removed -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        path -> Ltree,
        distinguished -> Bool,
        language_id -> Int4,
    }
}

diesel::table! {
    comment_aggregates (id) {
        id -> Int4,
        comment_id -> Int4,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        published -> Timestamp,
        child_count -> Int4,
    }
}

diesel::table! {
    comment_like (id) {
        id -> Int4,
        person_id -> Int4,
        comment_id -> Int4,
        post_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

diesel::table! {
    comment_reply (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamp,
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
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    comment_saved (id) {
        id -> Int4,
        comment_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    community (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        removed -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
        nsfw -> Bool,
        #[max_length = 255]
        actor_id -> Varchar,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamp,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 255]
        followers_url -> Varchar,
        #[max_length = 255]
        inbox_url -> Varchar,
        #[max_length = 255]
        shared_inbox_url -> Nullable<Varchar>,
        hidden -> Bool,
        posting_restricted_to_mods -> Bool,
        instance_id -> Int4,
        #[max_length = 255]
        moderators_url -> Nullable<Varchar>,
        #[max_length = 255]
        featured_url -> Nullable<Varchar>,
    }
}

diesel::table! {
    community_aggregates (id) {
        id -> Int4,
        community_id -> Int4,
        subscribers -> Int8,
        posts -> Int8,
        comments -> Int8,
        published -> Timestamp,
        users_active_day -> Int8,
        users_active_week -> Int8,
        users_active_month -> Int8,
        users_active_half_year -> Int8,
    }
}

diesel::table! {
    community_block (id) {
        id -> Int4,
        person_id -> Int4,
        community_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    community_follower (id) {
        id -> Int4,
        community_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
        pending -> Bool,
    }
}

diesel::table! {
    community_language (id) {
        id -> Int4,
        community_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    community_moderator (id) {
        id -> Int4,
        community_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    community_person_ban (id) {
        id -> Int4,
        community_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
        expires -> Nullable<Timestamp>,
    }
}

diesel::table! {
    custom_emoji (id) {
        id -> Int4,
        local_site_id -> Int4,
        #[max_length = 128]
        shortcode -> Varchar,
        image_url -> Text,
        alt_text -> Text,
        category -> Text,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    custom_emoji_keyword (id) {
        id -> Int4,
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
        published -> Timestamp,
    }
}

diesel::table! {
    federation_allowlist (id) {
        id -> Int4,
        instance_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    federation_blocklist (id) {
        id -> Int4,
        instance_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        #[max_length = 255]
        domain -> Varchar,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        #[max_length = 255]
        software -> Nullable<Varchar>,
        #[max_length = 255]
        version -> Nullable<Varchar>,
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
    use diesel::sql_types::{Bool, Int4, Nullable, Text, Timestamp, Varchar};
    use super::sql_types::ListingTypeEnum;
    use super::sql_types::RegistrationModeEnum;

    local_site (id) {
        id -> Int4,
        site_id -> Int4,
        site_setup -> Bool,
        enable_downvotes -> Bool,
        enable_nsfw -> Bool,
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
        federation_worker_count -> Int4,
        captcha_enabled -> Bool,
        #[max_length = 255]
        captcha_difficulty -> Varchar,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        registration_mode -> RegistrationModeEnum,
        reports_email_admins -> Bool,
    }
}

diesel::table! {
    local_site_rate_limit (id) {
        id -> Int4,
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
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::{Bool, Int4, Nullable, Text, Timestamp, Varchar};
    use super::sql_types::SortTypeEnum;
    use super::sql_types::ListingTypeEnum;

    local_user (id) {
        id -> Int4,
        person_id -> Int4,
        password_encrypted -> Text,
        email -> Nullable<Text>,
        show_nsfw -> Bool,
        #[max_length = 20]
        theme -> Varchar,
        default_sort_type -> SortTypeEnum,
        default_listing_type -> ListingTypeEnum,
        #[max_length = 20]
        interface_language -> Varchar,
        show_avatars -> Bool,
        send_notifications_to_email -> Bool,
        validator_time -> Timestamp,
        show_scores -> Bool,
        show_bot_accounts -> Bool,
        show_read_posts -> Bool,
        show_new_post_notifs -> Bool,
        email_verified -> Bool,
        accepted_application -> Bool,
        totp_2fa_secret -> Nullable<Text>,
        totp_2fa_url -> Nullable<Text>,
    }
}

diesel::table! {
    local_user_language (id) {
        id -> Int4,
        local_user_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    mod_add (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        removed -> Bool,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_add_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_ban (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Bool,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
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
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_feature_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        featured -> Bool,
        when_ -> Timestamp,
        is_featured_community -> Bool,
    }
}

diesel::table! {
    mod_hide_community (id) {
        id -> Int4,
        community_id -> Int4,
        mod_person_id -> Int4,
        when_ -> Timestamp,
        reason -> Nullable<Text>,
        hidden -> Bool,
    }
}

diesel::table! {
    mod_lock_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        locked -> Bool,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_comment (id) {
        id -> Int4,
        mod_person_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_remove_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Bool,
        when_ -> Timestamp,
    }
}

diesel::table! {
    mod_transfer_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        when_ -> Timestamp,
    }
}

diesel::table! {
    password_reset_request (id) {
        id -> Int4,
        token_encrypted -> Text,
        published -> Timestamp,
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
        banned -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        #[max_length = 255]
        actor_id -> Varchar,
        bio -> Nullable<Text>,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamp,
        banner -> Nullable<Text>,
        deleted -> Bool,
        #[max_length = 255]
        inbox_url -> Varchar,
        #[max_length = 255]
        shared_inbox_url -> Nullable<Varchar>,
        matrix_user_id -> Nullable<Text>,
        admin -> Bool,
        bot_account -> Bool,
        ban_expires -> Nullable<Timestamp>,
        instance_id -> Int4,
    }
}

diesel::table! {
    person_aggregates (id) {
        id -> Int4,
        person_id -> Int4,
        post_count -> Int8,
        post_score -> Int8,
        comment_count -> Int8,
        comment_score -> Int8,
    }
}

diesel::table! {
    person_ban (id) {
        id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    person_block (id) {
        id -> Int4,
        person_id -> Int4,
        target_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    person_follower (id) {
        id -> Int4,
        person_id -> Int4,
        follower_id -> Int4,
        published -> Timestamp,
        pending -> Bool,
    }
}

diesel::table! {
    person_mention (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamp,
    }
}

diesel::table! {
    person_post_aggregates (id) {
        id -> Int4,
        person_id -> Int4,
        post_id -> Int4,
        read_comments -> Int8,
        published -> Timestamp,
    }
}

diesel::table! {
    post (id) {
        id -> Int4,
        #[max_length = 200]
        name -> Varchar,
        url -> Nullable<Text>,
        body -> Nullable<Text>,
        creator_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        locked -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
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
    }
}

diesel::table! {
    post_aggregates (id) {
        id -> Int4,
        post_id -> Int4,
        comments -> Int8,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        published -> Timestamp,
        newest_comment_time_necro -> Timestamp,
        newest_comment_time -> Timestamp,
        featured_community -> Bool,
        featured_local -> Bool,
    }
}

diesel::table! {
    post_like (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

diesel::table! {
    post_read (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
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
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    post_saved (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
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
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
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
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    registration_application (id) {
        id -> Int4,
        local_user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published -> Timestamp,
    }
}

diesel::table! {
    secret (id) {
        id -> Int4,
        jwt_secret -> Varchar,
    }
}

diesel::table! {
    site (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
        sidebar -> Nullable<Text>,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        icon -> Nullable<Text>,
        banner -> Nullable<Text>,
        #[max_length = 150]
        description -> Nullable<Varchar>,
        #[max_length = 255]
        actor_id -> Varchar,
        last_refreshed_at -> Timestamp,
        #[max_length = 255]
        inbox_url -> Varchar,
        private_key -> Nullable<Text>,
        public_key -> Text,
        instance_id -> Int4,
    }
}

diesel::table! {
    site_aggregates (id) {
        id -> Int4,
        site_id -> Int4,
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
    site_language (id) {
        id -> Int4,
        site_id -> Int4,
        language_id -> Int4,
    }
}

diesel::table! {
    tagline (id) {
        id -> Int4,
        local_site_id -> Int4,
        content -> Text,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::joinable!(admin_purge_comment -> person (admin_person_id));
diesel::joinable!(admin_purge_comment -> post (post_id));
diesel::joinable!(admin_purge_community -> person (admin_person_id));
diesel::joinable!(admin_purge_person -> person (admin_person_id));
diesel::joinable!(admin_purge_post -> community (community_id));
diesel::joinable!(admin_purge_post -> person (admin_person_id));
diesel::joinable!(comment -> language (language_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(comment -> post (post_id));
diesel::joinable!(comment_aggregates -> comment (comment_id));
diesel::joinable!(comment_like -> comment (comment_id));
diesel::joinable!(comment_like -> person (person_id));
diesel::joinable!(comment_like -> post (post_id));
diesel::joinable!(comment_reply -> comment (comment_id));
diesel::joinable!(comment_reply -> person (recipient_id));
diesel::joinable!(comment_report -> comment (comment_id));
diesel::joinable!(comment_saved -> comment (comment_id));
diesel::joinable!(comment_saved -> person (person_id));
diesel::joinable!(community -> instance (instance_id));
diesel::joinable!(community_aggregates -> community (community_id));
diesel::joinable!(community_block -> community (community_id));
diesel::joinable!(community_block -> person (person_id));
diesel::joinable!(community_follower -> community (community_id));
diesel::joinable!(community_follower -> person (person_id));
diesel::joinable!(community_language -> community (community_id));
diesel::joinable!(community_language -> language (language_id));
diesel::joinable!(community_moderator -> community (community_id));
diesel::joinable!(community_moderator -> person (person_id));
diesel::joinable!(community_person_ban -> community (community_id));
diesel::joinable!(community_person_ban -> person (person_id));
diesel::joinable!(custom_emoji -> local_site (local_site_id));
diesel::joinable!(custom_emoji_keyword -> custom_emoji (custom_emoji_id));
diesel::joinable!(email_verification -> local_user (local_user_id));
diesel::joinable!(federation_allowlist -> instance (instance_id));
diesel::joinable!(federation_blocklist -> instance (instance_id));
diesel::joinable!(local_site -> site (site_id));
diesel::joinable!(local_site_rate_limit -> local_site (local_site_id));
diesel::joinable!(local_user -> person (person_id));
diesel::joinable!(local_user_language -> language (language_id));
diesel::joinable!(local_user_language -> local_user (local_user_id));
diesel::joinable!(mod_add_community -> community (community_id));
diesel::joinable!(mod_ban_from_community -> community (community_id));
diesel::joinable!(mod_feature_post -> person (mod_person_id));
diesel::joinable!(mod_feature_post -> post (post_id));
diesel::joinable!(mod_hide_community -> community (community_id));
diesel::joinable!(mod_hide_community -> person (mod_person_id));
diesel::joinable!(mod_lock_post -> person (mod_person_id));
diesel::joinable!(mod_lock_post -> post (post_id));
diesel::joinable!(mod_remove_comment -> comment (comment_id));
diesel::joinable!(mod_remove_comment -> person (mod_person_id));
diesel::joinable!(mod_remove_community -> community (community_id));
diesel::joinable!(mod_remove_community -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> person (mod_person_id));
diesel::joinable!(mod_remove_post -> post (post_id));
diesel::joinable!(mod_transfer_community -> community (community_id));
diesel::joinable!(password_reset_request -> local_user (local_user_id));
diesel::joinable!(person -> instance (instance_id));
diesel::joinable!(person_aggregates -> person (person_id));
diesel::joinable!(person_ban -> person (person_id));
diesel::joinable!(person_mention -> comment (comment_id));
diesel::joinable!(person_mention -> person (recipient_id));
diesel::joinable!(person_post_aggregates -> person (person_id));
diesel::joinable!(person_post_aggregates -> post (post_id));
diesel::joinable!(post -> community (community_id));
diesel::joinable!(post -> language (language_id));
diesel::joinable!(post -> person (creator_id));
diesel::joinable!(post_aggregates -> post (post_id));
diesel::joinable!(post_like -> person (person_id));
diesel::joinable!(post_like -> post (post_id));
diesel::joinable!(post_read -> person (person_id));
diesel::joinable!(post_read -> post (post_id));
diesel::joinable!(post_report -> post (post_id));
diesel::joinable!(post_saved -> person (person_id));
diesel::joinable!(post_saved -> post (post_id));
diesel::joinable!(private_message_report -> private_message (private_message_id));
diesel::joinable!(registration_application -> local_user (local_user_id));
diesel::joinable!(registration_application -> person (admin_id));
diesel::joinable!(site -> instance (instance_id));
diesel::joinable!(site_aggregates -> site (site_id));
diesel::joinable!(site_language -> language (language_id));
diesel::joinable!(site_language -> site (site_id));
diesel::joinable!(tagline -> local_site (local_site_id));

diesel::allow_tables_to_appear_in_same_query!(
    activity,
    admin_purge_comment,
    admin_purge_community,
    admin_purge_person,
    admin_purge_post,
    comment,
    comment_aggregates,
    comment_like,
    comment_reply,
    comment_report,
    comment_saved,
    community,
    community_aggregates,
    community_block,
    community_follower,
    community_language,
    community_moderator,
    community_person_ban,
    custom_emoji,
    custom_emoji_keyword,
    email_verification,
    federation_allowlist,
    federation_blocklist,
    instance,
    language,
    local_site,
    local_site_rate_limit,
    local_user,
    local_user_language,
    mod_add,
    mod_add_community,
    mod_ban,
    mod_ban_from_community,
    mod_feature_post,
    mod_hide_community,
    mod_lock_post,
    mod_remove_comment,
    mod_remove_community,
    mod_remove_post,
    mod_transfer_community,
    password_reset_request,
    person,
    person_aggregates,
    person_ban,
    person_block,
    person_follower,
    person_mention,
    person_post_aggregates,
    post,
    post_aggregates,
    post_like,
    post_read,
    post_report,
    post_saved,
    private_message,
    private_message_report,
    registration_application,
    secret,
    site,
    site_aggregates,
    site_language,
    tagline,
);
