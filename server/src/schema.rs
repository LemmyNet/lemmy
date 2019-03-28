table! {
    comment (id) {
        id -> Int4,
        content -> Text,
        attributed_to -> Text,
        post_id -> Int4,
        parent_id -> Nullable<Int4>,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

table! {
    comment_like (id) {
        id -> Int4,
        comment_id -> Int4,
        post_id -> Int4,
        fedi_user_id -> Text,
        score -> Int2,
        published -> Timestamp,
    }
}

table! {
    community (id) {
        id -> Int4,
        name -> Varchar,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

table! {
    community_follower (id) {
        id -> Int4,
        community_id -> Int4,
        fedi_user_id -> Text,
        published -> Timestamp,
    }
}

table! {
    community_user (id) {
        id -> Int4,
        community_id -> Int4,
        fedi_user_id -> Text,
        published -> Timestamp,
    }
}

table! {
    post (id) {
        id -> Int4,
        name -> Varchar,
        url -> Nullable<Text>,
        body -> Nullable<Text>,
        attributed_to -> Text,
        community_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

table! {
    post_like (id) {
        id -> Int4,
        post_id -> Int4,
        fedi_user_id -> Text,
        score -> Int2,
        published -> Timestamp,
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
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

joinable!(comment -> post (post_id));
joinable!(comment_like -> comment (comment_id));
joinable!(comment_like -> post (post_id));
joinable!(community_follower -> community (community_id));
joinable!(community_user -> community (community_id));
joinable!(post -> community (community_id));
joinable!(post_like -> post (post_id));

allow_tables_to_appear_in_same_query!(
    comment,
    comment_like,
    community,
    community_follower,
    community_user,
    post,
    post_like,
    user_,
);
