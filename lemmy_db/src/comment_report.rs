use diesel::{dsl::*, pg::Pg, result::Error, *};
use serde::{Deserialize, Serialize};

use crate::{limit_and_offset, MaybeOptional, schema::comment_report, comment::Comment, Reportable, naive_now};

table! {
    comment_report_view (id) {
      id -> Int4,
      creator_id -> Int4,
      comment_id -> Int4,
      comment_text -> Text,
      reason -> Text,
      resolved -> Bool,
      resolver_id -> Nullable<Int4>,
      published -> Timestamp,
      updated -> Nullable<Timestamp>,
      post_id -> Int4,
      community_id -> Int4,
      creator_name -> Varchar,
      comment_creator_id -> Int4,
      comment_creator_name -> Varchar,
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Comment)]
#[table_name = "comment_report"]
pub struct CommentReport {
  pub id: i32,
  pub creator_id: i32,
  pub comment_id: i32,
  pub comment_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<i32>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "comment_report"]
pub struct CommentReportForm {
  pub creator_id: i32,
  pub comment_id: i32,
  pub comment_text: String,
  pub reason: String,
}

impl Reportable<CommentReportForm> for CommentReport {
  fn report(conn: &PgConnection, comment_report_form: &CommentReportForm) -> Result<Self, Error> {
    use crate::schema::comment_report::dsl::*;
    insert_into(comment_report)
        .values(comment_report_form)
        .get_result::<Self>(conn)
  }

  fn resolve(conn: &PgConnection, report_id: i32, by_user_id: i32) -> Result<usize, Error> {
    use crate::schema::comment_report::dsl::*;
    update(comment_report.find(report_id))
        .set((
          resolved.eq(true),
          resolver_id.eq(by_user_id),
          updated.eq(naive_now()),
        ))
        .execute(conn)
  }

  fn unresolve(conn: &PgConnection, report_id: i32) -> Result<usize, Error> {
    use crate::schema::comment_report::dsl::*;
    update(comment_report.find(report_id))
        .set((
          resolved.eq(false),
          updated.eq(naive_now()),
        ))
        .execute(conn)
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, Clone,
)]
#[table_name = "comment_report_view"]
pub struct CommentReportView {
  pub id: i32,
  pub creator_id: i32,
  pub comment_id: i32,
  pub comment_text: String,
  pub reason: String,
  pub resolved: bool,
  pub resolver_id: Option<i32>,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub post_id: i32,
  pub community_id: i32,
  pub creator_name: String,
  pub comment_creator_id: i32,
  pub comment_creator_name: String,
}

pub struct CommentReportQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: comment_report_view::BoxedQuery<'a, Pg>,
  for_community_id: Option<i32>,
  page: Option<i64>,
  limit: Option<i64>,
  resolved: Option<bool>,
}

impl CommentReportView {
  pub fn read(conn: &PgConnection, report_id: i32) -> Result<Self, Error> {
    use super::comment_report::comment_report_view::dsl::*;
    comment_report_view
      .find(report_id)
      .first::<Self>(conn)
  }
}

impl<'a> CommentReportQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    use super::comment_report::comment_report_view::dsl::*;

    let query = comment_report_view.into_boxed();

    CommentReportQueryBuilder {
      conn,
      query,
      for_community_id: None,
      page: None,
      limit: None,
      resolved: Some(false),
    }
  }

  pub fn community_id<T: MaybeOptional<i32>>(mut self, community_id: T) -> Self {
    self.for_community_id = community_id.get_optional();
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

  pub fn list(self) -> Result<Vec<CommentReportView>, Error> {
    use super::comment_report::comment_report_view::dsl::*;

    let mut query = self.query;

    if let Some(comm_id) = self.for_community_id {
      query = query.filter(community_id.eq(comm_id));
    }

    if let Some(resolved_flag) = self.resolved {
      query = query.filter(resolved.eq(resolved_flag));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    query
      .order_by(published.asc())
      .limit(limit)
      .offset(offset)
      .load::<CommentReportView>(self.conn)
  }

  pub fn count(self) -> Result<usize, Error> {
    use super::comment_report::comment_report_view::dsl::*;
    let mut query = self.query;

    if let Some(comm_id) = self.for_community_id {
      query = query.filter(community_id.eq(comm_id));
    }

    if let Some(resolved_flag) = self.resolved {
      query = query.filter(resolved.eq(resolved_flag));
    }

    query.execute(self.conn)
  }
}


