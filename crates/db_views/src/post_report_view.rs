use crate::structs::{LocalUserView, PostReportView};
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
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
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PostReportView, (PostReportId, PersonId)>,
  impl ListFn<'a, PostReportView, (PostReportQuery, &'a LocalUserView)>,
> {
  let all_joins = |query: post_report::BoxedQuery<'a, Pg>, my_person_id: PersonId| {
    query
      .inner_join(post::table)
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(post_report::creator_id.eq(person::id)))
      .inner_join(aliases::person1.on(post::creator_id.eq(aliases::person1.field(person::id))))
      .left_join(
        community_person_ban::table.on(
          post::community_id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(post::creator_id)),
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
        aliases::person2
          .on(post_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      )
      .select((
        post_report::all_columns,
        post::all_columns,
        community::all_columns,
        person::all_columns,
        aliases::person1.fields(person::all_columns),
        community_person_ban::id.nullable().is_not_null(),
        post_like::score.nullable(),
        post_aggregates::all_columns,
        aliases::person2.fields(person::all_columns.nullable()),
      ))
  };

  let read = move |mut conn: DbConn<'a>, (report_id, my_person_id): (PostReportId, PersonId)| async move {
    all_joins(
      post_report::table.find(report_id).into_boxed(),
      my_person_id,
    )
    .first::<PostReportView>(&mut conn)
    .await
  };

  let list = move |mut conn: DbConn<'a>, (options, user): (PostReportQuery, &'a LocalUserView)| async move {
    let mut query = all_joins(post_report::table.into_boxed(), user.person.id);

    if let Some(community_id) = options.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if options.unresolved_only {
      query = query.filter(post_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query
      .order_by(post_report::published.desc())
      .limit(limit)
      .offset(offset);

    // If its not an admin, get only the ones you mod
    if !user.local_user.admin {
      query
        .inner_join(
          community_moderator::table.on(
            community_moderator::community_id
              .eq(post::community_id)
              .and(community_moderator::person_id.eq(user.person.id)),
          ),
        )
        .load::<PostReportView>(&mut conn)
        .await
    } else {
      query.load::<PostReportView>(&mut conn).await
    }
  };

  Queries::new(read, list)
}

impl PostReportView {
  /// returns the PostReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: PostReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    queries().read(pool, (report_id, my_person_id)).await
  }

  /// returns the current unresolved post report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    admin: bool,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;
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
        .await
    } else {
      query
        .select(count(post_report::id))
        .first::<i64>(conn)
        .await
    }
  }
}

#[derive(Default)]
pub struct PostReportQuery {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unresolved_only: bool,
}

impl PostReportQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> Result<Vec<PostReportView>, Error> {
    queries().list(pool, (self, user)).await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    post_report_view::{PostReportQuery, PostReportView},
    structs::LocalUserView,
  };
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostUpdateForm},
      post_report::{PostReport, PostReportForm},
    },
    traits::{Crud, Joinable, Reportable},
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_person = PersonInsertForm::builder()
      .name("timmy_prv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_timmy = Person::create(pool, &new_person).await.unwrap();

    let new_local_user = LocalUserInsertForm::builder()
      .person_id(inserted_timmy.id)
      .password_encrypted("123".to_string())
      .build();
    let timmy_local_user = LocalUser::create(pool, &new_local_user).await.unwrap();
    let timmy_view = LocalUserView {
      local_user: timmy_local_user,
      person: inserted_timmy.clone(),
      counts: Default::default(),
    };

    let new_person_2 = PersonInsertForm::builder()
      .name("sara_prv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_sara = Person::create(pool, &new_person_2).await.unwrap();

    // Add a third person, since new ppl can only report something once.
    let new_person_3 = PersonInsertForm::builder()
      .name("jessica_prv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_jessica = Person::create(pool, &new_person_3).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community prv".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    // Make timmy a mod
    let timmy_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_timmy.id,
    };

    let _inserted_moderator = CommunityModerator::join(pool, &timmy_moderator_form)
      .await
      .unwrap();

    let new_post = PostInsertForm::builder()
      .name("A test post crv".into())
      .creator_id(inserted_timmy.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    // sara reports
    let sara_report_form = PostReportForm {
      creator_id: inserted_sara.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from sara".into(),
    };

    PostReport::report(pool, &sara_report_form).await.unwrap();

    let new_post_2 = PostInsertForm::builder()
      .name("A test post crv 2".into())
      .creator_id(inserted_timmy.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post_2 = Post::create(pool, &new_post_2).await.unwrap();

    // jessica reports
    let jessica_report_form = PostReportForm {
      creator_id: inserted_jessica.id,
      post_id: inserted_post_2.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = PostReport::report(pool, &jessica_report_form)
      .await
      .unwrap();

    let read_jessica_report_view =
      PostReportView::read(pool, inserted_jessica_report.id, inserted_timmy.id)
        .await
        .unwrap();

    assert_eq!(
      read_jessica_report_view.post_report,
      inserted_jessica_report
    );
    assert_eq!(read_jessica_report_view.post, inserted_post_2);
    assert_eq!(read_jessica_report_view.community.id, inserted_community.id);
    assert_eq!(read_jessica_report_view.creator.id, inserted_jessica.id);
    assert_eq!(read_jessica_report_view.post_creator.id, inserted_timmy.id);
    assert_eq!(read_jessica_report_view.my_vote, None);
    assert_eq!(read_jessica_report_view.resolver, None);

    // Do a batch read of timmys reports
    let reports = PostReportQuery::default()
      .list(pool, &timmy_view)
      .await
      .unwrap();

    assert_eq!(reports[0].creator.id, inserted_jessica.id);
    assert_eq!(reports[1].creator.id, inserted_sara.id);

    // Make sure the counts are correct
    let report_count = PostReportView::get_report_count(pool, inserted_timmy.id, false, None)
      .await
      .unwrap();
    assert_eq!(2, report_count);

    // Removing post should automatically resolve the report via db trigger
    let removed_form = PostUpdateForm {
      removed: Some(true),
      ..Default::default()
    };
    Post::update(pool, inserted_jessica_report.post_id, &removed_form)
      .await
      .unwrap();

    let read_jessica_report_view_after_resolve =
      PostReportView::read(pool, inserted_jessica_report.id, inserted_timmy.id)
        .await
        .unwrap();
    assert!(read_jessica_report_view_after_resolve.post_report.resolved);
    // TODO: the db trigger has no way to set the resolver_id
    //assert_eq!(read_jessica_report_view_after_resolve.post_report.resolver_id, Some(inserted_timmy.id));
    //assert_eq!(read_jessica_report_view_after_resolve.resolver.unwrap().id, inserted_timmy.id);

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = PostReportQuery {
      unresolved_only: true,
      ..Default::default()
    }
    .list(pool, &timmy_view)
    .await
    .unwrap();
    assert_eq!(reports_after_resolve.len(), 1);
    assert_eq!(reports_after_resolve[0].creator.id, inserted_sara.id);

    // Make sure the counts are correct
    let report_count_after_resolved =
      PostReportView::get_report_count(pool, inserted_timmy.id, false, None)
        .await
        .unwrap();
    assert_eq!(1, report_count_after_resolved);

    Person::delete(pool, inserted_timmy.id).await.unwrap();
    Person::delete(pool, inserted_sara.id).await.unwrap();
    Person::delete(pool, inserted_jessica.id).await.unwrap();
    Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();
  }
}
