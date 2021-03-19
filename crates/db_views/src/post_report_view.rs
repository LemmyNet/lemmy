use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, MaybeOptional, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{community, person, person_alias_1, person_alias_2, post, post_report},
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonAlias1, PersonAlias2, PersonSafe, PersonSafeAlias1, PersonSafeAlias2},
    post::Post,
    post_report::PostReport,
  },
  CommunityId,
};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PostReportView {
  pub post_report: PostReport,
  pub post: Post,
  pub community: CommunitySafe,
  pub creator: PersonSafe,
  pub post_creator: PersonSafeAlias1,
  pub resolver: Option<PersonSafeAlias2>,
}

type PostReportViewTuple = (
  PostReport,
  Post,
  CommunitySafe,
  PersonSafe,
  PersonSafeAlias1,
  Option<PersonSafeAlias2>,
);

impl PostReportView {
  /// returns the PostReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(conn: &PgConnection, report_id: i32) -> Result<Self, Error> {
    let (post_report, post, community, creator, post_creator, resolver) = post_report::table
      .find(report_id)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(post::creator_id.eq(person_alias_1::id)))
      .left_join(
        person_alias_2::table.on(post_report::resolver_id.eq(person_alias_2::id.nullable())),
      )
      .select((
        post_report::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
        PersonAlias2::safe_columns_tuple().nullable(),
      ))
      .first::<PostReportViewTuple>(conn)?;

    Ok(Self {
      post_report,
      post,
      community,
      creator,
      post_creator,
      resolver,
    })
  }

  /// returns the current unresolved post report count for the supplied community ids
  ///
  /// * `community_ids` - a Vec<i32> of community_ids to get a count for
  /// TODO this eq_any is a bad way to do this, would be better to join to communitymoderator
  /// for a person id
  pub fn get_report_count(
    conn: &PgConnection,
    community_ids: &[CommunityId],
  ) -> Result<i64, Error> {
    use diesel::dsl::*;
    post_report::table
      .inner_join(post::table)
      .filter(
        post_report::resolved
          .eq(false)
          .and(post::community_id.eq_any(community_ids)),
      )
      .select(count(post_report::id))
      .first::<i64>(conn)
  }
}

pub struct PostReportQueryBuilder<'a> {
  conn: &'a PgConnection,
  community_ids: Option<Vec<CommunityId>>, // TODO bad way to do this
  page: Option<i64>,
  limit: Option<i64>,
  resolved: Option<bool>,
}

impl<'a> PostReportQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection) -> Self {
    PostReportQueryBuilder {
      conn,
      community_ids: None,
      page: None,
      limit: None,
      resolved: Some(false),
    }
  }

  pub fn community_ids<T: MaybeOptional<Vec<CommunityId>>>(mut self, community_ids: T) -> Self {
    self.community_ids = community_ids.get_optional();
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
    let mut query = post_report::table
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(post::creator_id.eq(person_alias_1::id)))
      .left_join(
        person_alias_2::table.on(post_report::resolver_id.eq(person_alias_2::id.nullable())),
      )
      .select((
        post_report::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
        PersonAlias2::safe_columns_tuple().nullable(),
      ))
      .into_boxed();

    if let Some(comm_ids) = self.community_ids {
      query = query.filter(post::community_id.eq_any(comm_ids));
    }

    if let Some(resolved_flag) = self.resolved {
      query = query.filter(post_report::resolved.eq(resolved_flag));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    let res = query
      .order_by(post_report::published.asc())
      .limit(limit)
      .offset(offset)
      .load::<PostReportViewTuple>(self.conn)?;

    Ok(PostReportView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PostReportView {
  type DbTuple = PostReportViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        post_report: a.0.to_owned(),
        post: a.1.to_owned(),
        community: a.2.to_owned(),
        creator: a.3.to_owned(),
        post_creator: a.4.to_owned(),
        resolver: a.5.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
