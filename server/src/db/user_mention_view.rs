use super::user_mention_view::user_mention_view::BoxedQuery;
use super::*;
use diesel::pg::Pg;

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
    creator_avatar -> Nullable<Text>,
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
  pub creator_avatar: Option<String>,
  pub score: i64,
  pub upvotes: i64,
  pub downvotes: i64,
  pub user_id: Option<i32>,
  pub my_vote: Option<i32>,
  pub saved: Option<bool>,
  pub recipient_id: i32,
}

pub struct UserMentionQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: BoxedQuery<'a, Pg>,
  for_user_id: i32,
  sort: &'a SortType,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserMentionQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, for_user_id: i32) -> Self {
    use super::user_mention_view::user_mention_view::dsl::*;

    let query = user_mention_view.into_boxed();

    UserMentionQueryBuilder {
      conn,
      query,
      for_user_id,
      sort: &SortType::New,
      unread_only: false,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn unread_only(mut self, unread_only: bool) -> Self {
    self.unread_only = unread_only;
    self
  }

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<UserMentionView>, Error> {
    use super::user_mention_view::user_mention_view::dsl::*;

    let mut query = self.query;

    if self.unread_only {
      query = query.filter(read.eq(false));
    }

    query = query
      .filter(user_id.eq(self.for_user_id))
      .filter(recipient_id.eq(self.for_user_id));

    query = match self.sort {
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

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    query
      .limit(limit)
      .offset(offset)
      .load::<UserMentionView>(self.conn)
  }
}

impl UserMentionView {
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
