use super::*;

// The faked schema since diesel doesn't do views
table! {
  user_mention_view (id) {
    id -> Int4,
    user_mention_id -> Int4,
    creator_id -> Int4,
    post_id -> Int4,
    parent_id -> Nullable<Int4>,
    content -> Text,
    removed -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    deleted -> Bool,
    community_id -> Int4,
    banned -> Bool,
    banned_from_community -> Bool,
    creator_name -> Varchar,
    score -> BigInt,
    upvotes -> BigInt,
    downvotes -> BigInt,
    user_id -> Nullable<Int4>,
    my_vote -> Nullable<Int4>,
    saved -> Nullable<Bool>,
    recipient_id -> Int4,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "user_mention_view"]
pub struct UserMentionView {
  pub id: i32,
  pub user_mention_id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub parent_id: Option<i32>,
  pub content: String,
  pub removed: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub community_id: i32,
  pub banned: bool,
  pub banned_from_community: bool,
  pub creator_name: String,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub saved: Option<bool>,
  pub recipient_id: i32,
}

impl UserMentionView {
  pub fn get_mentions(
    conn: &PgConnection,
    for_user_id: i32,
    sort: &SortType,
    unread_only: bool,
    page: Option<i64>,
    limit: Option<i64>,
  ) -> Result<Vec<Self>, Error> {
    use super::user_mention_view::user_mention_view::dsl::*;

    let (limit, offset) = limit_and_offset(page, limit);

    let mut query = user_mention_view.into_boxed();

    query = query
      .filter(user_id.eq(for_user_id))
      .filter(recipient_id.eq(for_user_id));

    if unread_only {
      query = query.filter(read.eq(false));
    }

    query = match sort {
      // SortType::Hot => query.order_by(hot_rank.desc()),
      SortType::New => query.order_by(published.desc()),
      SortType::TopAll => query.order_by(score.desc()),
      SortType::TopYear => query
        .filter(published.gt(now - 1.years()))
        .order_by(score.desc()),
      SortType::TopMonth => query
        .filter(published.gt(now - 1.months()))
        .order_by(score.desc()),
      SortType::TopWeek => query
        .filter(published.gt(now - 1.weeks()))
        .order_by(score.desc()),
      SortType::TopDay => query
        .filter(published.gt(now - 1.days()))
        .order_by(score.desc()),
      _ => query.order_by(published.desc()),
    };

    query.limit(limit).offset(offset).load::<Self>(conn)
  }

  pub fn read(
    conn: &PgConnection,
    from_user_mention_id: i32,
    from_recipient_id: i32,
  ) -> Result<Self, Error> {
    use super::user_mention_view::user_mention_view::dsl::*;

    user_mention_view
      .filter(user_mention_id.eq(from_user_mention_id))
      .filter(user_id.eq(from_recipient_id))
      .first::<Self>(conn)
  }
}
