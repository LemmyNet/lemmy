use super::user_view::user_view::BoxedQuery;
use super::*;
use diesel::pg::Pg;

table! {
  user_view (id) {
    id -> Int4,
    name -> Varchar,
    avatar -> Nullable<Text>,
    email -> Nullable<Text>,
    fedi_name -> Varchar,
    admin -> Bool,
    banned -> Bool,
    show_avatars -> Bool,
    send_notifications_to_email -> Bool,
    published -> Timestamp,
    number_of_posts -> BigInt,
    post_score -> BigInt,
    number_of_comments -> BigInt,
    comment_score -> BigInt,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "user_view"]
pub struct UserView {
  pub id: i32,
  pub name: String,
  pub avatar: Option<String>,
  pub email: Option<String>,
  pub fedi_name: String,
  pub admin: bool,
  pub banned: bool,
  pub show_avatars: bool,
  pub send_notifications_to_email: bool,
  pub published: chrono::NaiveDateTime,
  pub number_of_posts: i64,
  pub post_score: i64,
  pub number_of_comments: i64,
  pub comment_score: i64,
}

pub struct UserQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: BoxedQuery<'a, Pg>,
  sort: &'a SortType,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> UserQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    use super::user_view::user_view::dsl::*;

    let query = user_view.into_boxed();

    UserQueryBuilder {
      conn,
      query,
      sort: &SortType::Hot,
      page: None,
      limit: None,
    }
  }

  pub fn sort(mut self, sort: &'a SortType) -> Self {
    self.sort = sort;
    self
  }

  pub fn search_term<T: MaybeOptional<String>>(mut self, search_term: T) -> Self {
    use super::user_view::user_view::dsl::*;
    if let Some(search_term) = search_term.get_optional() {
      self.query = self.query.filter(name.ilike(fuzzy_search(&search_term)));
    }
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

  pub fn list(self) -> Result<Vec<UserView>, Error> {
    use super::user_view::user_view::dsl::*;

    let mut query = self.query;

    query = match self.sort {
      SortType::Hot => query
        .order_by(comment_score.desc())
        .then_order_by(published.desc()),
      SortType::New => query.order_by(published.desc()),
      SortType::TopAll => query.order_by(comment_score.desc()),
      SortType::TopYear => query
        .filter(published.gt(now - 1.years()))
        .order_by(comment_score.desc()),
      SortType::TopMonth => query
        .filter(published.gt(now - 1.months()))
        .order_by(comment_score.desc()),
      SortType::TopWeek => query
        .filter(published.gt(now - 1.weeks()))
        .order_by(comment_score.desc()),
      SortType::TopDay => query
        .filter(published.gt(now - 1.days()))
        .order_by(comment_score.desc()),
    };

    let (limit, offset) = limit_and_offset(self.page, self.limit);
    query = query.limit(limit).offset(offset);

    query.load::<UserView>(self.conn)
  }
}

impl UserView {
  pub fn read(conn: &PgConnection, from_user_id: i32) -> Result<Self, Error> {
    use super::user_view::user_view::dsl::*;

    user_view.find(from_user_id).first::<Self>(conn)
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view.filter(admin.eq(true)).load::<Self>(conn)
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view.filter(banned.eq(true)).load::<Self>(conn)
  }
}
