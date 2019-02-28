table! {
    community (id) {
        id -> Int4,
        name -> Varchar,
        starttime -> Timestamp,
    }
}

table! {
    community_user (id) {
        id -> Int4,
        fedi_user_id -> Varchar,
        community_id -> Nullable<Int4>,
        community_user_type -> Int2,
        starttime -> Timestamp,
    }
}

table! {
    user_ (id) {
        id -> Int4,
        name -> Varchar,
        password_encrypted -> Varchar,
        email -> Nullable<Varchar>,
        icon -> Nullable<Bytea>,
        starttime -> Timestamp,
    }
}

joinable!(community_user -> community (community_id));

allow_tables_to_appear_in_same_query!(
    community,
    community_user,
    user_,
);
