use super::*;

table! {
  user_view (id) {
    id -> Int4,
    name -> Varchar,
    fedi_name -> Varchar,
    admin -> Bool,
    banned -> Bool,
    published -> Timestamp,
    number_of_posts -> BigInt,
    post_score -> BigInt,
    number_of_comments -> BigInt,
    comment_score -> BigInt,
  }
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize,QueryableByName,Clone)]
#[table_name="user_view"]
pub struct UserView {
  pub id: i32,
  pub name: String,
  pub fedi_name: String,
  pub admin: bool,
  pub banned: bool,
  pub published: chrono::NaiveDateTime,
  pub number_of_posts: i64,
  pub post_score: i64,
  pub number_of_comments: i64,
  pub comment_score: i64,
}

impl UserView {

  pub fn list(conn: &PgConnection, 
              sort: &SortType, 
              search_term: Option<String>,
              page: Option<i64>,
              limit: Option<i64>,
              ) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;

    let (limit, offset) = limit_and_offset(page, limit);

    let mut query = user_view.into_boxed();

    if let Some(search_term) = search_term {
      query = query.filter(name.ilike(fuzzy_search(&search_term)));
    };

    query = match sort {
      SortType::Hot => query.order_by(comment_score.desc())
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
              .order_by(comment_score.desc())
    };

    query = query
      .limit(limit)
      .offset(offset);

    query.load::<Self>(conn) 
  }

  pub fn read(conn: &PgConnection, from_user_id: i32) -> Result<Self, Error> {
    use super::user_view::user_view::dsl::*;

    user_view.find(from_user_id)
    .first::<Self>(conn)
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view.filter(admin.eq(true))
    .load::<Self>(conn)
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_view::dsl::*;
    user_view.filter(banned.eq(true))
    .load::<Self>(conn)
  }
}

