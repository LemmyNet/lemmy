table! {
    community (id) {
        id -> Int4,
        name -> Varchar,
        start_time -> Timestamp,
    }
}

table! {
    community_follower (id) {
        id -> Int4,
        fedi_user_id -> Text,
        community_id -> Nullable<Int4>,
        start_time -> Timestamp,
    }
}

table! {
    community_user (id) {
        id -> Int4,
        fedi_user_id -> Text,
        community_id -> Nullable<Int4>,
        start_time -> Timestamp,
    }
}

table! {
    post (id) {
        id -> Int4,
        name -> Varchar,
        url -> Text,
        attributed_to -> Text,
        start_time -> Timestamp,
    }
}

table! {
    post_dislike (id) {
        id -> Int4,
        fedi_user_id -> Text,
        post_id -> Nullable<Int4>,
        start_time -> Timestamp,
    }
}

table! {
    post_like (id) {
        id -> Int4,
        fedi_user_id -> Text,
        post_id -> Nullable<Int4>,
        start_time -> Timestamp,
    }
}

table! {
    user_ (id) {
        id -> Int4,
        name -> Varchar,
        preferred_username -> Nullable<Varchar>,
        password_encrypted -> Text,
        email -> Nullable<Text>,
        icon -> Nullable<Bytea>,
        start_time -> Timestamp,
    }
}

joinable!(community_follower -> community (community_id));
joinable!(community_user -> community (community_id));
joinable!(post_dislike -> post (post_id));
joinable!(post_like -> post (post_id));

allow_tables_to_appear_in_same_query!(
    community,
    community_follower,
    community_user,
    post,
    post_dislike,
    post_like,
    user_,
);
