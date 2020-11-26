use diesel::{dsl::*, pg::Pg, result::Error, *};
use serde::{Deserialize, Serialize};

use crate::{
  limit_and_offset,
  naive_now,
  post::Post,
  schema::post_report,
  MaybeOptional,
  Reportable,
};

table! {
    post_report_view (id) {
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
        current_post_name -> Varchar,
        current_post_url -> Nullable<Text>,
        current_post_body -> Nullable<Text>,
        community_id -> Int4,
        creator_actor_id -> Text,
        creator_name -> Varchar,
        creator_preferred_username -> Nullable<Varchar>,
        creator_avatar -> Nullable<Text>,
        creator_local -> Bool,
        post_creator_id -> Int4,
        post_creator_actor_id -> Text,
        post_creator_name -> Varchar,
        post_creator_preferred_username -> Nullable<Varchar>,
        post_creator_avatar -> Nullable<Text>,
        post_creator_local -> Bool,
        resolver_actor_id -> Nullable<Text>,
        resolver_name -> Nullable<Varchar>,
        resolver_preferred_username -> Nullable<Varchar>,
        resolver_avatar -> Nullable<Text>,
        resolver_local -> Nullable<Bool>,
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Serialize, Deserialize, Debug)]
#[belongs_to(Post)]
#[table_name = "post_report"]
pub struct PostReport {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub original_post_name: String,
  pub original_post_url: Option<String>,
  pub original_post_body: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<i32>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "post_report"]
pub struct PostReportForm {
  pub creator_id: i32,
  pub post_id: i32,
  pub original_post_name: String,
  pub original_post_url: Option<String>,
  pub original_post_body: Option<String>,
  pub reason: String,
}

impl Reportable<PostReportForm> for PostReport {
  /// creates a post report and returns it
  ///
  /// * `conn` - the postgres connection
  /// * `post_report_form` - the filled CommentReportForm to insert
  fn report(conn: &PgConnection, post_report_form: &PostReportForm) -> Result<Self, Error> {
    use crate::schema::post_report::dsl::*;
    insert_into(post_report)
      .values(post_report_form)
      .get_result::<Self>(conn)
  }

  /// resolve a post report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to resolve
  /// * `by_resolver_id` - the id of the user resolving the report
  fn resolve(conn: &PgConnection, report_id: i32, by_resolver_id: i32) -> Result<usize, Error> {
    use crate::schema::post_report::dsl::*;
    update(post_report.find(report_id))
      .set((
        resolved.eq(true),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }

  /// resolve a post report
  ///
  /// * `conn` - the postgres connection
  /// * `report_id` - the id of the report to unresolve
  /// * `by_resolver_id` - the id of the user unresolving the report
  fn unresolve(conn: &PgConnection, report_id: i32, by_resolver_id: i32) -> Result<usize, Error> {
    use crate::schema::post_report::dsl::*;
    update(post_report.find(report_id))
      .set((
        resolved.eq(false),
        resolver_id.eq(by_resolver_id),
        updated.eq(naive_now()),
      ))
      .execute(conn)
  }
}

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[table_name = "post_report_view"]
pub struct PostReportView {
  pub id: i32,
  pub creator_id: i32,
  pub post_id: i32,
  pub original_post_name: String,
  pub original_post_url: Option<String>,
  pub original_post_body: Option<String>,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<i32>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub current_post_name: String,
  pub current_post_url: Option<String>,
  pub current_post_body: Option<String>,
  pub community_id: i32,
  pub creator_actor_id: String,
  pub creator_name: String,
  pub creator_preferred_username: Option<String>,
  pub creator_avatar: Option<String>,
  pub creator_local: bool,
  pub post_creator_id: i32,
  pub post_creator_actor_id: String,
  pub post_creator_name: String,
  pub post_creator_preferred_username: Option<String>,
  pub post_creator_avatar: Option<String>,
  pub post_creator_local: bool,
  pub resolver_actor_id: Option<String>,
  pub resolver_name: Option<String>,
  pub resolver_preferred_username: Option<String>,
  pub resolver_avatar: Option<String>,
  pub resolver_local: Option<bool>,
}

impl PostReportView {
  /// returns the PostReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(conn: &PgConnection, report_id: i32) -> Result<Self, Error> {
    use super::post_report::post_report_view::dsl::*;
    post_report_view.find(report_id).first::<Self>(conn)
  }

  /// returns the current unresolved post report count for the supplied community ids
  ///
  /// * `community_ids` - a Vec<i32> of community_ids to get a count for
  pub fn get_report_count(conn: &PgConnection, community_ids: &[i32]) -> Result<i64, Error> {
    use super::post_report::post_report_view::dsl::*;
    post_report_view
      .filter(resolved.eq(false).and(community_id.eq_any(community_ids)))
      .select(count(id))
      .first::<i64>(conn)
  }
}

pub struct PostReportQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: post_report_view::BoxedQuery<'a, Pg>,
  for_community_ids: Option<Vec<i32>>,
  page: Option<i64>,
  limit: Option<i64>,
  resolved: Option<bool>,
}

impl<'a> PostReportQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    use super::post_report::post_report_view::dsl::*;

    let query = post_report_view.into_boxed();

    PostReportQueryBuilder {
      conn,
      query,
      for_community_ids: None,
      page: None,
      limit: None,
      resolved: Some(false),
    }
  }

  pub fn community_ids<T: MaybeOptional<Vec<i32>>>(mut self, community_ids: T) -> Self {
    self.for_community_ids = community_ids.get_optional();
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

  pub fn resolved<T: MaybeOptional<bool>>(mut self, resolved: T) -> Self {
    self.resolved = resolved.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<PostReportView>, Error> {
    use super::post_report::post_report_view::dsl::*;

    let mut query = self.query;

    if let Some(comm_ids) = self.for_community_ids {
      query = query.filter(community_id.eq_any(comm_ids));
    }

    if let Some(resolved_flag) = self.resolved {
      query = query.filter(resolved.eq(resolved_flag));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    query
      .order_by(published.asc())
      .limit(limit)
      .offset(offset)
      .load::<PostReportView>(self.conn)
  }
}
