table! {
    activity (id) {
        id -> Int4,
        data -> Jsonb,
        local -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        ap_id -> Text,
        sensitive -> Nullable<Bool>,
    }
}

table! {
  use diesel_ltree::sql_types::Ltree;
  use diesel::sql_types::*;

    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        content -> Text,
        removed -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
        ap_id -> Varchar,
        local -> Bool,
        path -> Ltree,
        distinguished -> Bool,
        language_id -> Int4,
    }
}

table! {
    comment_aggregates (id) {
        id -> Int4,
        comment_id -> Int4,
        score -> Int8,
        upvotes -> Int8,
        downvotes -> Int8,
        published -> Timestamp,
        child_count ->  Int4,
    }
}

table! {
    comment_like (id) {
        id -> Int4,
        person_id -> Int4,
        comment_id -> Int4,
        post_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

table! {
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

table! {
    comment_saved (id) {
        id -> Int4,
        comment_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    community (id) {
        id -> Int4,
        name -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        removed -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        deleted -> Bool,
        nsfw -> Bool,
        actor_id -> Varchar,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamp,
        icon -> Nullable<Varchar>,
        banner -> Nullable<Varchar>,
        followers_url -> Varchar,
        inbox_url -> Varchar,
        shared_inbox_url -> Nullable<Varchar>,
        hidden -> Bool,
        posting_restricted_to_mods -> Bool,
        instance_id -> Int4,
    }
}

table! {
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

table! {
    community_follower (id) {
        id -> Int4,
        community_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
        pending -> Bool,
    }
}

table! {
    community_moderator (id) {
        id -> Int4,
        community_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    community_person_ban (id) {
        id -> Int4,
        community_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
        expires -> Nullable<Timestamp>,
    }
}

table! {
    local_user (id) {
        id -> Int4,
        person_id -> Int4,
        password_encrypted -> Text,
        email -> Nullable<Text>,
        show_nsfw -> Bool,
        theme -> Varchar,
        default_sort_type -> Int2,
        default_listing_type -> Int2,
        interface_language -> Varchar,
        show_avatars -> Bool,
        send_notifications_to_email -> Bool,
        validator_time -> Timestamp,
        show_bot_accounts -> Bool,
        show_scores -> Bool,
        show_read_posts -> Bool,
        show_new_post_notifs -> Bool,
        email_verified -> Bool,
        approved -> Bool,
    }
}

table! {
    mod_add (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    mod_add_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    mod_transfer_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    mod_ban (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

table! {
    mod_ban_from_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        other_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        banned -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

table! {
    mod_lock_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        locked -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    mod_remove_comment (id) {
        id -> Int4,
        mod_person_id -> Int4,
        comment_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    mod_remove_community (id) {
        id -> Int4,
        mod_person_id -> Int4,
        community_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        expires -> Nullable<Timestamp>,
        when_ -> Timestamp,
    }
}

table! {
    mod_remove_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        reason -> Nullable<Text>,
        removed -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    mod_feature_post (id) {
        id -> Int4,
        mod_person_id -> Int4,
        post_id -> Int4,
        featured -> Bool,
        when_ -> Timestamp,
        is_featured_community -> Bool,
    }
}

table! {
    password_reset_request (id) {
        id -> Int4,
        token_encrypted -> Text,
        published -> Timestamp,
        local_user_id -> Int4,
    }
}

table! {
    person (id) {
        id -> Int4,
        name -> Varchar,
        display_name -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        banned -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        actor_id -> Varchar,
        bio -> Nullable<Text>,
        local -> Bool,
        private_key -> Nullable<Text>,
        public_key -> Text,
        last_refreshed_at -> Timestamp,
        banner -> Nullable<Varchar>,
        deleted -> Bool,
        inbox_url -> Varchar,
        shared_inbox_url -> Nullable<Varchar>,
        matrix_user_id -> Nullable<Text>,
        admin -> Bool,
        bot_account -> Bool,
        ban_expires -> Nullable<Timestamp>,
        instance_id -> Int4,
    }
}

table! {
    person_aggregates (id) {
        id -> Int4,
        person_id -> Int4,
        post_count -> Int8,
        post_score -> Int8,
        comment_count -> Int8,
        comment_score -> Int8,
    }
}

table! {
    person_ban (id) {
        id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    person_mention (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamp,
    }
}

table! {
    comment_reply (id) {
        id -> Int4,
        recipient_id -> Int4,
        comment_id -> Int4,
        read -> Bool,
        published -> Timestamp,
    }
}

table! {
    post (id) {
        id -> Int4,
        name -> Varchar,
        url -> Nullable<Varchar>,
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
        embed_video_url -> Nullable<Text>,
        thumbnail_url -> Nullable<Text>,
        ap_id -> Varchar,
        local -> Bool,
        language_id -> Int4,
        featured_community -> Bool,
        featured_local -> Bool,
    }
}

table! {
    person_post_aggregates (id) {
        id -> Int4,
        person_id -> Int4,
        post_id -> Int4,
        read_comments -> Int8,
        published -> Timestamp,
    }
}

table! {
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

table! {
    post_like (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

table! {
    post_read (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    post_report (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
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

table! {
    post_saved (id) {
        id -> Int4,
        post_id -> Int4,
        person_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    private_message (id) {
        id -> Int4,
        creator_id -> Int4,
        recipient_id -> Int4,
        content -> Text,
        deleted -> Bool,
        read -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        ap_id -> Varchar,
        local -> Bool,
    }
}

table! {
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

table! {
    site (id) {
        id -> Int4,
        name -> Varchar,
        sidebar -> Nullable<Text>,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
        icon -> Nullable<Varchar>,
        banner -> Nullable<Varchar>,
        description -> Nullable<Text>,
        actor_id -> Text,
        last_refreshed_at -> Timestamp,
        inbox_url -> Text,
        private_key -> Nullable<Text>,
        public_key -> Text,
        instance_id -> Int4,
    }
}

table! {
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

table! {
    person_block (id) {
        id -> Int4,
        person_id -> Int4,
        target_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    community_block (id) {
        id -> Int4,
        person_id -> Int4,
        community_id -> Int4,
        published -> Timestamp,
    }
}

table! {
  secret(id) {
    id -> Int4,
    jwt_secret -> Varchar,
  }
}

table! {
  admin_purge_comment (id) {
    id -> Int4,
    admin_person_id -> Int4,
    post_id -> Int4,
    reason -> Nullable<Text>,
    when_ -> Timestamp,
  }
}

table! {
  email_verification (id) {
    id -> Int4,
    local_user_id -> Int4,
    email -> Text,
    verification_token -> Varchar,
    published -> Timestamp,
  }
}

table! {
  admin_purge_community (id) {
    id -> Int4,
    admin_person_id -> Int4,
    reason -> Nullable<Text>,
    when_ -> Timestamp,
  }
}

table! {
  admin_purge_person (id) {
    id -> Int4,
    admin_person_id -> Int4,
    reason -> Nullable<Text>,
    when_ -> Timestamp,
  }
}

table! {
  admin_purge_post (id) {
    id -> Int4,
    admin_person_id -> Int4,
    community_id -> Int4,
    reason -> Nullable<Text>,
    when_ -> Timestamp,
  }
}

table! {
    registration_application (id) {
        id -> Int4,
        local_user_id -> Int4,
        answer -> Text,
        admin_id -> Nullable<Int4>,
        deny_reason -> Nullable<Text>,
        published -> Timestamp,
    }
}

table! {
    mod_hide_community (id) {
        id -> Int4,
        community_id -> Int4,
        mod_person_id -> Int4,
        reason -> Nullable<Text>,
        hidden -> Nullable<Bool>,
        when_ -> Timestamp,
    }
}

table! {
    language (id) {
        id -> Int4,
        code -> Text,
        name -> Text,
    }
}

table! {
    local_user_language(id) {
        id -> Int4,
        local_user_id -> Int4,
        language_id -> Int4,
    }
}

table! {
    site_language(id) {
        id -> Int4,
        site_id -> Int4,
        language_id -> Int4,
    }
}

table! {
    community_language(id) {
        id -> Int4,
        community_id -> Int4,
        language_id -> Int4,
    }
}

table! {
  instance(id) {
    id -> Int4,
    domain -> Text,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
  }
}

table! {
  federation_allowlist(id) {
    id -> Int4,
    instance_id -> Int4,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
  }
}

table! {
  federation_blocklist(id) {
    id -> Int4,
    instance_id -> Int4,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
  }
}

table! {
  use crate::source::local_site::RegistrationModeType;
  use diesel::sql_types::*;

  local_site(id) {
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
    default_post_listing_type -> Text,
    legal_information -> Nullable<Text>,
    hide_modlog_mod_names -> Bool,
    application_email_admins -> Bool,
    slur_filter_regex -> Nullable<Text>,
    actor_name_max_length -> Int4,
    federation_enabled -> Bool,
    federation_debug -> Bool,
    federation_worker_count -> Int4,
    captcha_enabled -> Bool,
    captcha_difficulty -> Text,
    registration_mode -> RegistrationModeType,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
  }
}

table! {
  local_site_rate_limit(id) {
    id -> Int4,
    local_site_id -> Int4,
    message -> Int4,
    message_per_second-> Int4,
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

table! {
  tagline(id) {
    id -> Int4,
    local_site_id -> Int4,
    content -> Text,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
  }
}

table! {
    person_follower (id) {
        id -> Int4,
        person_id -> Int4,
        follower_id -> Int4,
        published -> Timestamp,
        pending -> Bool,
    }
}

table! {
    review_comment (id) {
        id -> Int4,
        comment_id -> Int4,
        approved -> Bool,
        approver_id -> Nullable<Int4>,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

joinable!(person_block -> person (person_id));

joinable!(comment -> person (creator_id));
joinable!(comment -> post (post_id));
joinable!(comment_aggregates -> comment (comment_id));
joinable!(comment_like -> comment (comment_id));
joinable!(comment_like -> person (person_id));
joinable!(comment_like -> post (post_id));
joinable!(comment_report -> comment (comment_id));
joinable!(comment_saved -> comment (comment_id));
joinable!(comment_saved -> person (person_id));
joinable!(community_aggregates -> community (community_id));
joinable!(community_block -> community (community_id));
joinable!(community_block -> person (person_id));
joinable!(community_follower -> community (community_id));
joinable!(community_follower -> person (person_id));
joinable!(community_moderator -> community (community_id));
joinable!(community_moderator -> person (person_id));
joinable!(community_person_ban -> community (community_id));
joinable!(community_person_ban -> person (person_id));
joinable!(local_user -> person (person_id));
joinable!(mod_add_community -> community (community_id));
joinable!(mod_transfer_community -> community (community_id));
joinable!(mod_ban_from_community -> community (community_id));
joinable!(mod_lock_post -> person (mod_person_id));
joinable!(mod_lock_post -> post (post_id));
joinable!(mod_remove_comment -> comment (comment_id));
joinable!(mod_remove_comment -> person (mod_person_id));
joinable!(mod_remove_community -> community (community_id));
joinable!(mod_remove_community -> person (mod_person_id));
joinable!(mod_remove_post -> person (mod_person_id));
joinable!(mod_remove_post -> post (post_id));
joinable!(mod_feature_post -> person (mod_person_id));
joinable!(mod_feature_post -> post (post_id));
joinable!(password_reset_request -> local_user (local_user_id));
joinable!(person_aggregates -> person (person_id));
joinable!(person_ban -> person (person_id));
joinable!(person_mention -> comment (comment_id));
joinable!(person_mention -> person (recipient_id));
joinable!(comment_reply -> comment (comment_id));
joinable!(comment_reply -> person (recipient_id));
joinable!(post -> community (community_id));
joinable!(post -> person (creator_id));
joinable!(person_post_aggregates -> post (post_id));
joinable!(person_post_aggregates -> person (person_id));
joinable!(post_aggregates -> post (post_id));
joinable!(post_like -> person (person_id));
joinable!(post_like -> post (post_id));
joinable!(post_read -> person (person_id));
joinable!(post_read -> post (post_id));
joinable!(post_report -> post (post_id));
joinable!(post_saved -> person (person_id));
joinable!(post_saved -> post (post_id));
joinable!(site_aggregates -> site (site_id));
joinable!(email_verification -> local_user (local_user_id));
joinable!(registration_application -> local_user (local_user_id));
joinable!(registration_application -> person (admin_id));
joinable!(mod_hide_community -> person (mod_person_id));
joinable!(mod_hide_community -> community (community_id));
joinable!(post -> language (language_id));
joinable!(comment -> language (language_id));
joinable!(local_user_language -> language (language_id));
joinable!(local_user_language -> local_user (local_user_id));
joinable!(private_message_report -> private_message (private_message_id));
joinable!(site_language -> language (language_id));
joinable!(site_language -> site (site_id));
joinable!(community_language -> language (language_id));
joinable!(community_language -> community (community_id));
joinable!(person_follower -> person (follower_id));
joinable!(review_comment -> comment (comment_id));
joinable!(review_comment -> local_user (approver_id));

joinable!(admin_purge_comment -> person (admin_person_id));
joinable!(admin_purge_comment -> post (post_id));
joinable!(admin_purge_community -> person (admin_person_id));
joinable!(admin_purge_person -> person (admin_person_id));
joinable!(admin_purge_post -> community (community_id));
joinable!(admin_purge_post -> person (admin_person_id));

joinable!(site -> instance (instance_id));
joinable!(person -> instance (instance_id));
joinable!(community -> instance (instance_id));
joinable!(federation_allowlist -> instance (instance_id));
joinable!(federation_blocklist -> instance (instance_id));
joinable!(local_site -> site (site_id));
joinable!(local_site_rate_limit -> local_site (local_site_id));
joinable!(tagline -> local_site (local_site_id));

allow_tables_to_appear_in_same_query!(
  activity,
  comment,
  comment_aggregates,
  community_block,
  comment_like,
  comment_report,
  comment_saved,
  community,
  community_aggregates,
  community_follower,
  community_moderator,
  community_person_ban,
  local_user,
  mod_add,
  mod_add_community,
  mod_transfer_community,
  mod_ban,
  mod_ban_from_community,
  mod_lock_post,
  mod_remove_comment,
  mod_remove_community,
  mod_remove_post,
  mod_feature_post,
  mod_hide_community,
  password_reset_request,
  person,
  person_aggregates,
  person_ban,
  person_block,
  person_mention,
  person_post_aggregates,
  comment_reply,
  post,
  post_aggregates,
  post_like,
  post_read,
  post_report,
  post_saved,
  private_message,
  private_message_report,
  site,
  site_aggregates,
  admin_purge_comment,
  admin_purge_community,
  admin_purge_person,
  admin_purge_post,
  email_verification,
  registration_application,
  language,
  tagline,
  local_user_language,
  site_language,
  community_language,
  instance,
  federation_allowlist,
  federation_blocklist,
  local_site,
  local_site_rate_limit,
  person_follower,
  review_comment
);
