use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, MaybeOptional, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{
    comment,
    comment_report,
    community,
    community_moderator,
    person,
    person_alias_1,
    person_alias_2,
    post,
  },
  source::{
    comment::Comment,
    comment_report::CommentReport,
    community::{Community, CommunitySafe},
    person::{Person, PersonAlias1, PersonAlias2, PersonSafe, PersonSafeAlias1, PersonSafeAlias2},
    post::Post,
  },
  CommunityId,
  PersonId,
};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct CommentReportView {
  pub comment_report: CommentReport,
  pub comment: Comment,
  pub post: Post,
  pub community: CommunitySafe,
  pub creator: PersonSafe,
  pub comment_creator: PersonSafeAlias1,
  pub resolver: Option<PersonSafeAlias2>,
}

type CommentReportViewTuple = (
  CommentReport,
  Comment,
  Post,
  CommunitySafe,
  PersonSafe,
  PersonSafeAlias1,
  Option<PersonSafeAlias2>,
);

impl CommentReportView {
  /// returns the CommentReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(conn: &PgConnection, report_id: i32) -> Result<Self, Error> {
    let (comment_report, comment, post, community, creator, comment_creator, resolver) =
      comment_report::table
        .find(report_id)
        .inner_join(comment::table)
        .inner_join(post::table.on(comment::post_id.eq(post::id)))
        .inner_join(community::table.on(post::community_id.eq(community::id)))
        .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
        .inner_join(person_alias_1::table.on(post::creator_id.eq(person_alias_1::id)))
        .left_join(
          person_alias_2::table.on(comment_report::resolver_id.eq(person_alias_2::id.nullable())),
        )
        .select((
          comment_report::all_columns,
          comment::all_columns,
          post::all_columns,
          Community::safe_columns_tuple(),
          Person::safe_columns_tuple(),
          PersonAlias1::safe_columns_tuple(),
          PersonAlias2::safe_columns_tuple().nullable(),
        ))
        .first::<CommentReportViewTuple>(conn)?;

    Ok(Self {
      comment_report,
      comment,
      post,
      community,
      creator,
      comment_creator,
      resolver,
    })
  }

  /// returns the current unresolved post report count for the supplied community ids
  ///
  /// * `community_ids` - a Vec<i32> of community_ids to get a count for
  /// TODO this eq_any is a bad way to do this, would be better to join to communitymoderator
  /// TODO FIX THIS NOW
  /// for a person id
  pub fn get_report_count(
    conn: &PgConnection,
    my_person_id: PersonId,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::*;

    let mut query = comment_report::table
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      // Test this join
      .inner_join(
        community_moderator::table.on(
          community_moderator::community_id
            .eq(post::community_id)
            .and(community_moderator::person_id.eq(my_person_id)),
        ),
      )
      .filter(comment_report::resolved.eq(false))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id))
    }

    query.select(count(comment_report::id)).first::<i64>(conn)
  }
}

pub struct CommentReportQueryBuilder<'a> {
  conn: &'a PgConnection,
  my_person_id: PersonId,
  community_id: Option<CommunityId>,
  page: Option<i64>,
  limit: Option<i64>,
  resolved: Option<bool>,
}

impl<'a> CommentReportQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, my_person_id: PersonId) -> Self {
    CommentReportQueryBuilder {
      conn,
      my_person_id,
      community_id: None,
      page: None,
      limit: None,
      resolved: Some(false),
    }
  }

  pub fn community_id<T: MaybeOptional<CommunityId>>(mut self, community_id: T) -> Self {
    self.community_id = community_id.get_optional();
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
    let mut query = comment_report::table
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(post::creator_id.eq(person_alias_1::id)))
      // Test this join
      .inner_join(
        community_moderator::table.on(
          community_moderator::community_id
            .eq(post::community_id)
            .and(community_moderator::person_id.eq(self.my_person_id)),
        ),
      )
      .left_join(
        person_alias_2::table.on(comment_report::resolver_id.eq(person_alias_2::id.nullable())),
      )
      .select((
        comment_report::all_columns,
        comment::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
        PersonAlias2::safe_columns_tuple().nullable(),
      ))
      .into_boxed();

    if let Some(community_id) = self.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if let Some(resolved_flag) = self.resolved {
      query = query.filter(comment_report::resolved.eq(resolved_flag));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    let res = query
      .order_by(comment_report::published.asc())
      .limit(limit)
      .offset(offset)
      .load::<CommentReportViewTuple>(self.conn)?;

    Ok(CommentReportView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommentReportView {
  type DbTuple = CommentReportViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        comment_report: a.0.to_owned(),
        comment: a.1.to_owned(),
        post: a.2.to_owned(),
        community: a.3.to_owned(),
        creator: a.4.to_owned(),
        comment_creator: a.5.to_owned(),
        resolver: a.6.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
