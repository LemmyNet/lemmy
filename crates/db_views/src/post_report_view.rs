use diesel::{result::Error, *};
use lemmy_db_queries::{limit_and_offset, MaybeOptional, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{
    community,
    community_moderator,
    person,
    person_alias_1,
    person_alias_2,
    post,
    post_report,
  },
  source::{
    community::{Community, CommunitySafe},
    person::{Person, PersonAlias1, PersonAlias2, PersonSafe, PersonSafeAlias1, PersonSafeAlias2},
    post::Post,
    post_report::PostReport,
  },
  CommunityId,
  PersonId,
  PostReportId,
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
  pub fn read(conn: &PgConnection, report_id: PostReportId) -> Result<Self, Error> {
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
    my_person_id: PersonId,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::*;
    let mut query = post_report::table
      .inner_join(post::table)
      // Test this join
      .inner_join(
        community_moderator::table.on(
          community_moderator::community_id
            .eq(post::community_id)
            .and(community_moderator::person_id.eq(my_person_id)),
        ),
      )
      .filter(post_report::resolved.eq(false))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id))
    }

    query.select(count(post_report::id)).first::<i64>(conn)
  }
}

pub struct PostReportQueryBuilder<'a> {
  conn: &'a PgConnection,
  my_person_id: PersonId,
  community_id: Option<CommunityId>,
  page: Option<i64>,
  limit: Option<i64>,
  resolved: bool,
}

impl<'a> PostReportQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, my_person_id: PersonId) -> Self {
    PostReportQueryBuilder {
      conn,
      my_person_id,
      community_id: None,
      page: None,
      limit: None,
      resolved: false,
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

  pub fn resolved(mut self, resolved: bool) -> Self {
    self.resolved = resolved;
    self
  }

  pub fn list(self) -> Result<Vec<PostReportView>, Error> {
    let mut query = post_report::table
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
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

    if let Some(community_id) = self.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    query = query.filter(post_report::resolved.eq(self.resolved));

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

#[cfg(test)]
mod tests {
  use crate::post_report_view::{PostReportQueryBuilder, PostReportView};
  use lemmy_db_queries::{establish_unpooled_connection, Crud, Joinable, Reportable};
  use lemmy_db_schema::source::{
    community::*,
    person::*,
    post::*,
    post_report::{PostReport, PostReportForm},
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "timmy_prv".into(),
      ..PersonForm::default()
    };

    let inserted_timmy = Person::create(&conn, &new_person).unwrap();

    let new_person_2 = PersonForm {
      name: "sara_prv".into(),
      ..PersonForm::default()
    };

    let inserted_sara = Person::create(&conn, &new_person_2).unwrap();

    // Add a third person, since new ppl can only report something once.
    let new_person_3 = PersonForm {
      name: "jessica_prv".into(),
      ..PersonForm::default()
    };

    let inserted_jessica = Person::create(&conn, &new_person_3).unwrap();

    let new_community = CommunityForm {
      name: "test community prv".to_string(),
      title: "nada".to_owned(),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    // Make timmy a mod
    let timmy_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_timmy.id,
    };

    let _inserted_moderator = CommunityModerator::join(&conn, &timmy_moderator_form).unwrap();

    let new_post = PostForm {
      name: "A test post crv".into(),
      creator_id: inserted_timmy.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(&conn, &new_post).unwrap();

    // sara reports
    let sara_report_form = PostReportForm {
      creator_id: inserted_sara.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
    };

    let inserted_sara_report = PostReport::report(&conn, &sara_report_form).unwrap();

    // jessica reports
    let jessica_report_form = PostReportForm {
      creator_id: inserted_jessica.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = PostReport::report(&conn, &jessica_report_form).unwrap();

    let read_jessica_report_view = PostReportView::read(&conn, inserted_jessica_report.id).unwrap();
    let expected_jessica_report_view = PostReportView {
      post_report: inserted_jessica_report.to_owned(),
      post: inserted_post,
      community: CommunitySafe {
        id: inserted_community.id,
        name: inserted_community.name,
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: inserted_community.actor_id.to_owned(),
        local: true,
        title: inserted_community.title,
        description: None,
        updated: None,
        banner: None,
        published: inserted_community.published,
      },
      creator: PersonSafe {
        id: inserted_jessica.id,
        name: inserted_jessica.name,
        display_name: None,
        published: inserted_jessica.published,
        avatar: None,
        actor_id: inserted_jessica.actor_id.to_owned(),
        local: true,
        banned: false,
        deleted: false,
        admin: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_jessica.inbox_url.to_owned(),
        shared_inbox_url: None,
        matrix_user_id: None,
      },
      post_creator: PersonSafeAlias1 {
        id: inserted_timmy.id,
        name: inserted_timmy.name.to_owned(),
        display_name: None,
        published: inserted_timmy.published,
        avatar: None,
        actor_id: inserted_timmy.actor_id.to_owned(),
        local: true,
        banned: false,
        deleted: false,
        admin: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_timmy.inbox_url.to_owned(),
        shared_inbox_url: None,
        matrix_user_id: None,
      },
      resolver: None,
    };

    assert_eq!(read_jessica_report_view, expected_jessica_report_view);

    let mut expected_sara_report_view = expected_jessica_report_view.clone();
    expected_sara_report_view.post_report = inserted_sara_report;
    expected_sara_report_view.creator = PersonSafe {
      id: inserted_sara.id,
      name: inserted_sara.name,
      display_name: None,
      published: inserted_sara.published,
      avatar: None,
      actor_id: inserted_sara.actor_id.to_owned(),
      local: true,
      banned: false,
      deleted: false,
      admin: false,
      bot_account: false,
      bio: None,
      banner: None,
      updated: None,
      inbox_url: inserted_sara.inbox_url.to_owned(),
      shared_inbox_url: None,
      matrix_user_id: None,
    };

    // Do a batch read of timmys reports
    let reports = PostReportQueryBuilder::create(&conn, inserted_timmy.id)
      .list()
      .unwrap();

    assert_eq!(
      reports,
      [
        expected_sara_report_view.to_owned(),
        expected_jessica_report_view.to_owned()
      ]
    );

    // Make sure the counts are correct
    let report_count = PostReportView::get_report_count(&conn, inserted_timmy.id, None).unwrap();
    assert_eq!(2, report_count);

    // Try to resolve the report
    PostReport::resolve(&conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();
    let read_jessica_report_view_after_resolve =
      PostReportView::read(&conn, inserted_jessica_report.id).unwrap();

    let mut expected_jessica_report_view_after_resolve = expected_jessica_report_view;
    expected_jessica_report_view_after_resolve
      .post_report
      .resolved = true;
    expected_jessica_report_view_after_resolve
      .post_report
      .resolver_id = Some(inserted_timmy.id);
    expected_jessica_report_view_after_resolve
      .post_report
      .updated = read_jessica_report_view_after_resolve.post_report.updated;
    expected_jessica_report_view_after_resolve.resolver = Some(PersonSafeAlias2 {
      id: inserted_timmy.id,
      name: inserted_timmy.name.to_owned(),
      display_name: None,
      published: inserted_timmy.published,
      avatar: None,
      actor_id: inserted_timmy.actor_id.to_owned(),
      local: true,
      banned: false,
      deleted: false,
      admin: false,
      bot_account: false,
      bio: None,
      banner: None,
      updated: None,
      inbox_url: inserted_timmy.inbox_url.to_owned(),
      shared_inbox_url: None,
      matrix_user_id: None,
    });

    assert_eq!(
      read_jessica_report_view_after_resolve,
      expected_jessica_report_view_after_resolve
    );

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = PostReportQueryBuilder::create(&conn, inserted_timmy.id)
      .list()
      .unwrap();
    assert_eq!(reports_after_resolve[0], expected_sara_report_view);

    // Make sure the counts are correct
    let report_count_after_resolved =
      PostReportView::get_report_count(&conn, inserted_timmy.id, None).unwrap();
    assert_eq!(1, report_count_after_resolved);

    Person::delete(&conn, inserted_timmy.id).unwrap();
    Person::delete(&conn, inserted_sara.id).unwrap();
    Person::delete(&conn, inserted_jessica.id).unwrap();
    Community::delete(&conn, inserted_community.id).unwrap();
  }
}
