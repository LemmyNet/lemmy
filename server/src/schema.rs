table! {
    category (id) {
        id -> Int4,
        name -> Varchar,
    }
}

table! {
    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        post_id -> Int4,
        parent_id -> Nullable<Int4>,
        content -> Text,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

table! {
    comment_like (id) {
        id -> Int4,
        user_id -> Int4,
        comment_id -> Int4,
        post_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

table! {
    community (id) {
        id -> Int4,
        name -> Varchar,
        title -> Varchar,
        description -> Nullable<Text>,
        category_id -> Int4,
        creator_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

table! {
    community_follower (id) {
        id -> Int4,
        community_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    community_moderator (id) {
        id -> Int4,
        community_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

table! {
    post (id) {
        id -> Int4,
        name -> Varchar,
        url -> Nullable<Text>,
        body -> Nullable<Text>,
        creator_id -> Int4,
        community_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

table! {
    post_like (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

table! {
    user_ (id) {
        id -> Int4,
        name -> Varchar,
        fedi_name -> Varchar,
        preferred_username -> Nullable<Varchar>,
        password_encrypted -> Text,
        email -> Nullable<Text>,
        icon -> Nullable<Bytea>,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

joinable!(comment -> post (post_id));
joinable!(comment -> user_ (creator_id));
joinable!(comment_like -> comment (comment_id));
joinable!(comment_like -> post (post_id));
joinable!(comment_like -> user_ (user_id));
joinable!(community -> category (category_id));
joinable!(community -> user_ (creator_id));
joinable!(community_follower -> community (community_id));
joinable!(community_follower -> user_ (user_id));
joinable!(community_moderator -> community (community_id));
joinable!(community_moderator -> user_ (user_id));
joinable!(post -> community (community_id));
joinable!(post -> user_ (creator_id));
joinable!(post_like -> post (post_id));
joinable!(post_like -> user_ (user_id));

allow_tables_to_appear_in_same_query!(
    category,
    comment,
    comment_like,
    community,
    community_follower,
    community_moderator,
    post,
    post_like,
    user_,
);
