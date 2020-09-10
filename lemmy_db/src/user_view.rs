use super::user_view::user_fast::BoxedQuery;
use crate::{fuzzy_search, limit_and_offset, MaybeOptional, SortType};
use diesel::{dsl::*, pg::Pg, result::Error, *};
use serde::{Serialize};

table! {
  user_view (id) {
    id -> Int4,
    actor_id -> Text,
    name -> Varchar,
    preferred_username -> Nullable<Varchar>,
    avatar -> Nullable<Text>,
    banner -> Nullable<Text>,
    email -> Nullable<Text>,
    matrix_user_id -> Nullable<Text>,
    bio -> Nullable<Text>,
    local -> Bool,
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

table! {
  user_fast (id) {
    id -> Int4,
    actor_id -> Text,
    name -> Varchar,
    preferred_username -> Nullable<Varchar>,
    avatar -> Nullable<Text>,
    banner -> Nullable<Text>,
    email -> Nullable<Text>,
    matrix_user_id -> Nullable<Text>,
    bio -> Nullable<Text>,
    local -> Bool,
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
  Queryable, Identifiable, PartialEq, Debug, Serialize, QueryableByName, Clone,
)]
#[table_name = "user_fast"]
pub struct UserView {
  pub id: i32,
  pub actor_id: String,
  pub name: String,
  pub preferred_username: Option<String>,
  pub avatar: Option<String>,
  pub banner: Option<String>,
  pub email: Option<String>, // TODO this shouldn't be in this view
  pub matrix_user_id: Option<String>,
  pub bio: Option<String>,
  pub local: bool,
  pub admin: bool,
  pub banned: bool,
  pub show_avatars: bool, // TODO this is a setting, probably doesn't need to be here
  pub send_notifications_to_email: bool, // TODO also never used
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
    use super::user_view::user_fast::dsl::*;

    let query = user_fast.into_boxed();

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
    use super::user_view::user_fast::dsl::*;
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
    use super::user_view::user_fast::dsl::*;
    use diesel::sql_types::{Nullable, Text};

    let mut query = self.query;

    query = match self.sort {
      SortType::Hot => query
        .order_by(comment_score.desc())
        .then_order_by(published.desc()),
      SortType::Active => query
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

    // The select is necessary here to not get back emails
    query = query.select((
      id,
      actor_id,
      name,
      preferred_username,
      avatar,
      banner,
      "".into_sql::<Nullable<Text>>(),
      matrix_user_id,
      bio,
      local,
      admin,
      banned,
      show_avatars,
      send_notifications_to_email,
      published,
      number_of_posts,
      post_score,
      number_of_comments,
      comment_score,
    ));
    query.load::<UserView>(self.conn)
  }
}

impl UserView {
  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_fast::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_fast
      // The select is necessary here to not get back emails
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
      ))
      .filter(admin.eq(true))
      .order_by(published)
      .load::<Self>(conn)
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use super::user_view::user_fast::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_fast
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
      ))
      .filter(banned.eq(true))
      .load::<Self>(conn)
  }

  pub fn get_user_secure(conn: &PgConnection, user_id: i32) -> Result<Self, Error> {
    use super::user_view::user_fast::dsl::*;
    use diesel::sql_types::{Nullable, Text};
    user_fast
      .select((
        id,
        actor_id,
        name,
        preferred_username,
        avatar,
        banner,
        "".into_sql::<Nullable<Text>>(),
        matrix_user_id,
        bio,
        local,
        admin,
        banned,
        show_avatars,
        send_notifications_to_email,
        published,
        number_of_posts,
        post_score,
        number_of_comments,
        comment_score,
      ))
      .find(user_id)
      .first::<Self>(conn)
  }
}
