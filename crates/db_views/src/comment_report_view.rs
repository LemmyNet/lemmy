use crate::structs::CommentReportView;
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  aggregates::structs::CommentAggregates,
  newtypes::{CommentReportId, CommunityId, PersonId},
  schema::{
    comment,
    comment_aggregates,
    comment_like,
    comment_report,
    community,
    community_moderator,
    community_person_ban,
    person,
    post,
  },
  source::{
    comment::Comment,
    comment_report::CommentReport,
    community::{Community, CommunityPersonBan, CommunitySafe},
    person::{Person, PersonSafe},
    post::Post,
  },
  traits::{ToSafe, ViewToVec},
  utils::limit_and_offset,
};
use typed_builder::TypedBuilder;

type CommentReportViewTuple = (
  CommentReport,
  Comment,
  Post,
  CommunitySafe,
  PersonSafe,
  PersonSafe,
  CommentAggregates,
  Option<CommunityPersonBan>,
  Option<i16>,
  Option<PersonSafe>,
);

impl CommentReportView {
  /// returns the CommentReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub fn read(
    conn: &mut PgConnection,
    report_id: CommentReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

    let (
      comment_report,
      comment,
      post,
      community,
      creator,
      comment_creator,
      counts,
      creator_banned_from_community,
      comment_like,
      resolver,
    ) = comment_report::table
      .find(report_id)
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1.on(comment::creator_id.eq(person_alias_1.field(person::id))))
      .inner_join(
        comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id)),
      )
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(my_person_id)),
        ),
      )
      .left_join(
        person_alias_2
          .on(comment_report::resolver_id.eq(person_alias_2.field(person::id).nullable())),
      )
      .select((
        comment_report::all_columns,
        comment::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        comment_like::score.nullable(),
        person_alias_2
          .fields(Person::safe_columns_tuple())
          .nullable(),
      ))
      .first::<CommentReportViewTuple>(conn)?;

    let my_vote = comment_like;

    Ok(Self {
      comment_report,
      comment,
      post,
      community,
      creator,
      comment_creator,
      counts,
      creator_banned_from_community: creator_banned_from_community.is_some(),
      my_vote,
      resolver,
    })
  }

  /// Returns the current unresolved post report count for the communities you mod
  pub fn get_report_count(
    conn: &mut PgConnection,
    my_person_id: PersonId,
    admin: bool,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::*;

    let mut query = comment_report::table
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .filter(comment_report::resolved.eq(false))
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
        .select(count(comment_report::id))
        .first::<i64>(conn)
    } else {
      query.select(count(comment_report::id)).first::<i64>(conn)
    }
  }
}

#[derive(TypedBuilder)]
#[builder(field_defaults(default))]
pub struct CommentReportQuery<'a> {
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

impl<'a> CommentReportQuery<'a> {
  pub fn list(self) -> Result<Vec<CommentReportView>, Error> {
    let (person_alias_1, person_alias_2) = diesel::alias!(person as person1, person as person2);

    let mut query = comment_report::table
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
      .inner_join(person_alias_1.on(comment::creator_id.eq(person_alias_1.field(person::id))))
      .inner_join(
        comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id)),
      )
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(comment::creator_id))
            .and(
              community_person_ban::expires
                .is_null()
                .or(community_person_ban::expires.gt(now)),
            ),
        ),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(self.my_person_id)),
        ),
      )
      .left_join(
        person_alias_2
          .on(comment_report::resolver_id.eq(person_alias_2.field(person::id).nullable())),
      )
      .select((
        comment_report::all_columns,
        comment::all_columns,
        post::all_columns,
        Community::safe_columns_tuple(),
        Person::safe_columns_tuple(),
        person_alias_1.fields(Person::safe_columns_tuple()),
        comment_aggregates::all_columns,
        community_person_ban::all_columns.nullable(),
        comment_like::score.nullable(),
        person_alias_2
          .fields(Person::safe_columns_tuple())
          .nullable(),
      ))
      .into_boxed();

    if let Some(community_id) = self.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if self.unresolved_only.unwrap_or(true) {
      query = query.filter(comment_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit)?;

    query = query
      .order_by(comment_report::published.desc())
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
        .load::<CommentReportViewTuple>(self.conn)?
    } else {
      query.load::<CommentReportViewTuple>(self.conn)?
    };

    Ok(CommentReportView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for CommentReportView {
  type DbTuple = CommentReportViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .into_iter()
      .map(|a| Self {
        comment_report: a.0,
        comment: a.1,
        post: a.2,
        community: a.3,
        creator: a.4,
        comment_creator: a.5,
        counts: a.6,
        creator_banned_from_community: a.7.is_some(),
        my_vote: a.8,
        resolver: a.9,
      })
      .collect::<Vec<Self>>()
  }
}

#[cfg(test)]
mod tests {
  use crate::comment_report_view::{CommentReportQuery, CommentReportView};
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    source::{comment::*, comment_report::*, community::*, person::*, post::*},
    traits::{Crud, Joinable, Reportable},
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let new_person = PersonForm {
      name: "timmy_crv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_timmy = Person::create(conn, &new_person).unwrap();

    let new_person_2 = PersonForm {
      name: "sara_crv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_sara = Person::create(conn, &new_person_2).unwrap();

    // Add a third person, since new ppl can only report something once.
    let new_person_3 = PersonForm {
      name: "jessica_crv".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_jessica = Person::create(conn, &new_person_3).unwrap();

    let new_community = CommunityForm {
      name: "test community crv".to_string(),
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

    let comment_form = CommentForm {
      content: "A test comment 32".into(),
      creator_id: inserted_timmy.id,
      post_id: inserted_post.id,
      ..CommentForm::default()
    };

    let inserted_comment = Comment::create(conn, &comment_form, None).unwrap();

    // sara reports
    let sara_report_form = CommentReportForm {
      creator_id: inserted_sara.id,
      comment_id: inserted_comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from sara".into(),
    };

    let inserted_sara_report = CommentReport::report(conn, &sara_report_form).unwrap();

    // jessica reports
    let jessica_report_form = CommentReportForm {
      creator_id: inserted_jessica.id,
      comment_id: inserted_comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = CommentReport::report(conn, &jessica_report_form).unwrap();

    let agg = CommentAggregates::read(conn, inserted_comment.id).unwrap();

    let read_jessica_report_view =
      CommentReportView::read(conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();
    let expected_jessica_report_view = CommentReportView {
      comment_report: inserted_jessica_report.to_owned(),
      comment: inserted_comment.to_owned(),
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
      comment_creator: PersonSafe {
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
      counts: CommentAggregates {
        id: agg.id,
        comment_id: inserted_comment.id,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        published: agg.published,
        child_count: 0,
      },
      my_vote: None,
      resolver: None,
    };

    assert_eq!(read_jessica_report_view, expected_jessica_report_view);

    let mut expected_sara_report_view = expected_jessica_report_view.clone();
    expected_sara_report_view.comment_report = inserted_sara_report;
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
    let reports = CommentReportQuery::builder()
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
      CommentReportView::get_report_count(conn, inserted_timmy.id, false, None).unwrap();
    assert_eq!(2, report_count);

    // Try to resolve the report
    CommentReport::resolve(conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();
    let read_jessica_report_view_after_resolve =
      CommentReportView::read(conn, inserted_jessica_report.id, inserted_timmy.id).unwrap();

    let mut expected_jessica_report_view_after_resolve = expected_jessica_report_view;
    expected_jessica_report_view_after_resolve
      .comment_report
      .resolved = true;
    expected_jessica_report_view_after_resolve
      .comment_report
      .resolver_id = Some(inserted_timmy.id);
    expected_jessica_report_view_after_resolve
      .comment_report
      .updated = read_jessica_report_view_after_resolve
      .comment_report
      .updated;
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
    let reports_after_resolve = CommentReportQuery::builder()
      .conn(conn)
      .my_person_id(inserted_timmy.id)
      .admin(false)
      .build()
      .list()
      .unwrap();
    assert_eq!(reports_after_resolve[0], expected_sara_report_view);
    assert_eq!(reports_after_resolve.len(), 1);

    // Make sure the counts are correct
    let report_count_after_resolved =
      CommentReportView::get_report_count(conn, inserted_timmy.id, false, None).unwrap();
    assert_eq!(1, report_count_after_resolved);

    Person::delete(conn, inserted_timmy.id).unwrap();
    Person::delete(conn, inserted_sara.id).unwrap();
    Person::delete(conn, inserted_jessica.id).unwrap();
    Community::delete(conn, inserted_community.id).unwrap();
  }
}
