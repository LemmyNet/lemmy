use diesel::{PgConnection, QueryDsl, RunQueryDsl, ExpressionMethods, insert_into, update};
use diesel::pg::Pg;
use diesel::result::*;
use serde::{Deserialize, Serialize};

use crate::{
    limit_and_offset,
    MaybeOptional,
    schema::post_report,
    post::Post,
    Reportable,
};

table! {
    post_report_view (id) {
      id -> Uuid,
      time -> Timestamp,
      reason -> Nullable<Text>,
      resolved -> Bool,
      user_id -> Int4,
      post_id -> Int4,
      post_name -> Varchar,
      post_url -> Nullable<Text>,
      post_body -> Nullable<Text>,
      post_time -> Timestamp,
      community_id -> Int4,
      user_name -> Varchar,
      creator_id -> Int4,
      creator_name -> Varchar,
    }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Serialize, Deserialize, Debug)]
#[belongs_to(Post)]
#[table_name = "post_report"]
pub struct PostReport {
    pub id: uuid::Uuid,
    pub time: chrono::NaiveDateTime,
    pub reason: Option<String>,
    pub resolved: bool,
    pub user_id: i32,
    pub post_id: i32,
    pub post_name: String,
    pub post_url: Option<String>,
    pub post_body: Option<String>,
    pub post_time: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "post_report"]
pub struct PostReportForm {
    pub time: Option<chrono::NaiveDateTime>,
    pub reason: Option<String>,
    pub resolved: Option<bool>,
    pub user_id: i32,
    pub post_id: i32,
    pub post_name: String,
    pub post_url: Option<String>,
    pub post_body: Option<String>,
    pub post_time: chrono::NaiveDateTime,
}

impl Reportable<PostReportForm> for PostReport {
    fn report(conn: &PgConnection, post_report_form: &PostReportForm) -> Result<Self, Error> {
        use crate::schema::post_report::dsl::*;
        insert_into(post_report)
            .values(post_report_form)
            .get_result::<Self>(conn)
    }

    fn resolve(conn: &PgConnection, report_id: &uuid::Uuid) -> Result<usize, Error> {
        use crate::schema::post_report::dsl::*;
        update(post_report.find(report_id))
            .set(resolved.eq(true))
            .execute(conn)
    }
}

#[derive(
Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "post_report_view"]
pub struct PostReportView {
    pub id: uuid::Uuid,
    pub time: chrono::NaiveDateTime,
    pub reason: Option<String>,
    pub resolved: bool,
    pub user_id: i32,
    pub post_id: i32,
    pub post_name: String,
    pub post_url: Option<String>,
    pub post_body: Option<String>,
    pub post_time: chrono::NaiveDateTime,
    pub community_id: i32,
    pub user_name: String,
    pub creator_id: i32,
    pub creator_name: String,
}

impl PostReportView {
    pub fn read(conn: &PgConnection, report_id: &uuid::Uuid) -> Result<Self, Error> {
        use super::post_report::post_report_view::dsl::*;
        post_report_view
            .filter(id.eq(report_id))
            .first::<Self>(conn)
    }
}

pub struct PostReportQueryBuilder<'a> {
    conn: &'a PgConnection,
    query: post_report_view::BoxedQuery<'a, Pg>,
    for_community_id: Option<i32>,
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

    pub fn list(self) -> Result<Vec<PostReportView>, Error> {
        use super::post_report::post_report_view::dsl::*;

        let mut query = self.query;

        if let Some(comm_id) = self.for_community_id {
            query = query.filter(community_id.eq(comm_id));
        }

        if let Some(resolved_flag) = self.resolved {
            query = query.filter(resolved.eq(resolved_flag));
        }

        let (limit, offset) = limit_and_offset(self.page, self.limit);

        query
            .order_by(time.desc())
            .limit(limit)
            .offset(offset)
            .load::<PostReportView>(self.conn)
    }

    pub fn count(self) -> Result<usize, Error> {
        use super::post_report::post_report_view::dsl::*;
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
