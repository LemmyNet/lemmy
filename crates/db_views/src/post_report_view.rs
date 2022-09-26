use crate::structs::PostReportView;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::PostAggregates,
  newtypes::{CommunityId, PersonId, PostReportId},
  schema::{
    community,
    community_moderator,
    community_person_ban,
    person,
    post,
    post_aggregates,
    post_like,
    post_report,
  },
  source::{
    community::{Community, CommunityPersonBan, CommunitySafe},
    person::{Person, PersonSafe},
    post::Post,
    post_report::PostReport,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};
use typed_builder::TypedBuilder;

type PostReportViewTuple = (
  PostReport,
  Post,
  CommunitySafe,
  PersonSafe,
  PersonSafe,
  Option<CommunityPersonBan>,
  Option<i16>,
  PostAggregates,
  Option<PersonSafe>,
);

impl PostReportView {
  /// returns the PostReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(
    conn: &mut PgConnection,
    report_id: PostReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

    let (
      post_report,
      post,
      community,
      creator,
      post_creator,
      creator_banned_from_community,
      post_like,
      counts,
      resolver,
    ) = post_report::table
      .find(report_id)
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1.on(post::creator_id.eq(person_alias_1.field(person::id))))
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::person_id.eq(my_person_id)),
        ),
      )
      .inner_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
      .left_join(
        person_alias_2.on(post_report::resolver_id.eq(person_alias_2.field(person::id).nullable())),
      )
      .select((
        post_report::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        community_person_ban::all_columns.nullable(),
        post_like::score.nullable(),
        post_aggregates::all_columns,
        person_alias_2.fields(Person::safe_columns_tuple().nullable()),
      ))
      .first::<PostReportViewTuple>(conn)?;

    let my_vote = post_like;

    Ok(Self {
      post_report,
      post,
      community,
      creator,
      post_creator,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      my_vote,
      counts,
      resolver,
    })
  }

  /// returns the current unresolved post report count for the communities you mod
  pub fn get_report_count(
    conn: &mut PgConnection,
    my_person_id: PersonId,
    admin: bool,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::*;
    let mut query = post_report::table
      .inner_join(post::table)
      .filter(post_report::resolved.eq(false))
      .into_boxed();

    if let Some(community_id) = community_id {
      query = query.filter(post::community_id.eq(community_id))
    }

    // If its not an admin, get only the ones you mod
    if !admin {
      query
        .inner_join(
          community_moderator::table.on(
            community_moderator::community_id
              .eq(post::community_id)
              .and(community_moderator::person_id.eq(my_person_id)),
          ),
        )
        .select(count(post_report::id))
        .first::<i64>(conn)
    } else {
      query.select(count(post_report::id)).first::<i64>(conn)
    }
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct PostReportQuery<'a> {
  #[builder(!default)]
  conn: &'a mut PgConnection,
  #[builder(!default)]
  my_person_id: PersonId,
  #[builder(!default)]
  admin: bool,
  community_id: Option<CommunityId>,
  page: Option<i64>,
  limit: Option<i64>,
  unresolved_only: Option<bool>,
}

impl<'a> PostReportQuery<'a> {
  pub fn list(self) -> Result<Vec<PostReportView>, Error> {
    let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

    let mut query = post_report::table
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1.on(post::creator_id.eq(person_alias_1.field(person::id))))
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        post_like::table.on(
          post::id
            .eq(post_like::post_id)
            .and(post_like::person_id.eq(self.my_person_id)),
        ),
      )
      .inner_join(post_aggregates::table.on(post_report::post_id.eq(post_aggregates::post_id)))
      .left_join(
        person_alias_2.on(post_report::resolver_id.eq(person_alias_2.field(person::id).nullable())),
      )
      .select((
        post_report::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        community_person_ban::all_columns.nullable(),
        post_like::score.nullable(),
        post_aggregates::all_columns,
        person_alias_2
          .fields(Person::safe_columns_tuple())
          .nullable(),
      ))
      .into_boxed();

    if let Some(community_id) = self.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if self.unresolved_only.unwrap_or(true) {
      query = query.filter(post_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    query = query
      .order_by(post_report::published.desc())
      .limit(limit)
      .offset(offset);

    // If its not an admin, get only the ones you mod
    let res = if !self.admin {
      query
        .inner_join(
          community_moderator::table.on(
            community_moderator::community_id
              .eq(post::community_id)
              .and(community_moderator::person_id.eq(self.my_person_id)),
          ),
        )
        .load::<PostReportViewTuple>(self.conn)?
    } else {
      query.load::<PostReportViewTuple>(self.conn)?
    };

    Ok(PostReportView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PostReportView {
  type DbTuple = PostReportViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        post_report: a.0,
        post: a.1,
        community: a.2,
        creator: a.3,
        post_creator: a.4,
        creator_banned_from_community: a.5.is_some(),
        my_vote: a.6,
        counts: a.7,
        resolver: a.8,
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::post_report_view::{PostReportQuery, PostReportView};
  use lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    source::{
      community::*,
      person::*,
      post::*,
      post_report::{PostReport, PostReportForm},
    },
    traits::{Crud, Joinable, Reportable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let new_person = PersonForm {
      name: "timmy_prv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_timmy = Person::create(conn, &new_person).unwrap();

    let new_person_2 = PersonForm {
      name: "sara_prv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_sara = Person::create(conn, &new_person_2).unwrap();

    // Add a third person, since new ppl can only report something once.
    let new_person_3 = PersonForm {
      name: "jessica_prv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_jessica = Person::create(conn, &new_person_3).unwrap();

    let new_community = CommunityForm {
      name: "test community prv".to_string(),
      title: "nada".to_owned(),
      public_key: Some("pubkey".to_string()),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(conn, &new_community).unwrap();

    // Make timmy a mod
    let timmy_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_timmy.id,
    };

    let _inserted_moderator = CommunityModerator::join(conn, &timmy_moderator_form).unwrap();

    let new_post = PostForm {
      name: "A test post crv".into(),
      creator_id: inserted_timmy.id,
      community_id: inserted_community.id,
      ..PostForm::default()
    };

    let inserted_post = Post::create(conn, &new_post).unwrap();

    // sara reports
    let sara_report_form = PostReportForm {
      creator_id: inserted_sara.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
    };

    let inserted_sara_report = PostReport::report(conn, &sara_report_form).unwrap();

    // jessica reports
    let jessica_report_form = PostReportForm {
      creator_id: inserted_jessica.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = PostReport::report(conn, &jessica_report_form).unwrap();

    let agg = PostAggregates::read(conn, inserted_post.id).unwrap();

    let read_jessica_report_view =
      PostReportView::read(conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();
    let expected_jessica_report_view = PostReportView {
      post_report: inserted_jessica_report.to_owned(),
      post: inserted_post.to_owned(),
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
        hidden: false,
        posting_restricted_to_mods: false,
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
        ban_expires: None,
      },
      post_creator: PersonSafe {
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
        ban_expires: None,
      },
      creator_banned_from_community: false,
      my_vote: None,
      counts: PostAggregates {
        id: agg.id,
        post_id: inserted_post.id,
        comments: 0,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        stickied: false,
        published: agg.published,
        newest_comment_time_necro: inserted_post.published,
        newest_comment_time: inserted_post.published,
      },
      resolver: None,
    };

    assert_eq!(read_jessica_report_view, expected_jessica_report_view);

    let mut expected_sara_report_view = expected_jessica_report_view.clone();
    expected_sara_report_view.post_report = inserted_sara_report;
    expected_sara_report_view.my_vote = None;
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
      ban_expires: None,
    };

    // Do a batch read of timmys reports
    let reports = PostReportQuery::builder()
      .conn(conn)
      .my_person_id(inserted_timmy.id)
      .admin(false)
      .build()
      .list()
      .unwrap();

    assert_eq!(
      reports,
      [
        expected_jessica_report_view.to_owned(),
        expected_sara_report_view.to_owned()
      ]
    );

    // Make sure the counts are correct
    let report_count =
      PostReportView::get_report_count(conn, inserted_timmy.id, false, None).unwrap();
    assert_eq!(2, report_count);

    // Try to resolve the report
    PostReport::resolve(conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();
    let read_jessica_report_view_after_resolve =
      PostReportView::read(conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();

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
    expected_jessica_report_view_after_resolve.resolver = Some(PersonSafe {
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
      ban_expires: None,
    });

    assert_eq!(
      read_jessica_report_view_after_resolve,
      expected_jessica_report_view_after_resolve
    );

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = PostReportQuery::builder()
      .conn(conn)
      .my_person_id(inserted_timmy.id)
      .admin(false)
      .build()
      .list()
      .unwrap();
    assert_eq!(reports_after_resolve[0], expected_sara_report_view);

    // Make sure the counts are correct
    let report_count_after_resolved =
      PostReportView::get_report_count(conn, inserted_timmy.id, false, None).unwrap();
    assert_eq!(1, report_count_after_resolved);

    Person::delete(conn, inserted_timmy.id).unwrap();
    Person::delete(conn, inserted_sara.id).unwrap();
    Person::delete(conn, inserted_jessica.id).unwrap();
    Community::delete(conn, inserted_community.id).unwrap();
  }
}
