use crate::structs::{CommentReportView, LocalUserView};
use diesel::{
  dsl::now,
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
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommentReportView, (CommentReportId, PersonId)>,
  impl ListFn<'a, CommentReportView, (CommentReportQuery, &'a LocalUserView)>,
> {
  let all_joins = |query: comment_report::BoxedQuery<'a, Pg>, my_person_id: PersonId| {
    query
      .inner_join(comment::table)
      .inner_join(post::table.on(comment::post_id.eq(post::id)))
      .inner_join(community::table.on(post::community_id.eq(community::id)))
      .inner_join(person::table.on(comment_report::creator_id.eq(person::id)))
      .inner_join(aliases::person1.on(comment::creator_id.eq(aliases::person1.field(person::id))))
      .inner_join(
        comment_aggregates::table.on(comment_report::comment_id.eq(comment_aggregates::comment_id)),
      )
      .left_join(
        comment_like::table.on(
          comment::id
            .eq(comment_like::comment_id)
            .and(comment_like::person_id.eq(my_person_id)),
        ),
      )
      .left_join(
        aliases::person2
          .on(comment_report::resolver_id.eq(aliases::person2.field(person::id).nullable())),
      )
  };

  let selection = (
    comment_report::all_columns,
    comment::all_columns,
    post::all_columns,
    community::all_columns,
    person::all_columns,
    aliases::person1.fields(person::all_columns),
    comment_aggregates::all_columns,
    community_person_ban::community_id.nullable().is_not_null(),
    comment_like::score.nullable(),
    aliases::person2.fields(person::all_columns).nullable(),
  );

  let read = move |mut conn: DbConn<'a>, (report_id, my_person_id): (CommentReportId, PersonId)| async move {
    all_joins(
      comment_report::table.find(report_id).into_boxed(),
      my_person_id,
    )
    .left_join(
      community_person_ban::table.on(
        community::id
          .eq(community_person_ban::community_id)
          .and(community_person_ban::person_id.eq(comment::creator_id)),
      ),
    )
    .select(selection)
    .first::<CommentReportView>(&mut conn)
    .await
  };

  let list = move |mut conn: DbConn<'a>,
                   (options, user): (CommentReportQuery, &'a LocalUserView)| async move {
    let mut query = all_joins(comment_report::table.into_boxed(), user.person.id)
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
      .select(selection);

    if let Some(community_id) = options.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if options.unresolved_only {
      query = query.filter(comment_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query
      .order_by(comment_report::published.desc())
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
        .load::<CommentReportView>(&mut conn)
        .await
    } else {
      query.load::<CommentReportView>(&mut conn).await
    }
  };

  Queries::new(read, list)
}

impl CommentReportView {
  /// returns the CommentReportView for the provided report_id
  ///
  /// * `report_id` - the report id to obtain
  pub async fn read(
    pool: &mut DbPool<'_>,
    report_id: CommentReportId,
    my_person_id: PersonId,
  ) -> Result<Self, Error> {
    queries().read(pool, (report_id, my_person_id)).await
  }

  /// Returns the current unresolved post report count for the communities you mod
  pub async fn get_report_count(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
    admin: bool,
    community_id: Option<CommunityId>,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;

    let conn = &mut get_conn(pool).await?;

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
        .await
    } else {
      query
        .select(count(comment_report::id))
        .first::<i64>(conn)
        .await
    }
  }
}

#[derive(Default)]
pub struct CommentReportQuery {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub unresolved_only: bool,
}

impl CommentReportQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    user: &LocalUserView,
  ) -> Result<Vec<CommentReportView>, Error> {
    queries().list(pool, (self, user)).await
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{
    comment_report_view::{CommentReportQuery, CommentReportView},
    structs::LocalUserView,
  };
  use lemmy_db_schema::{
    aggregates::structs::CommentAggregates,
    source::{
      comment::{Comment, CommentInsertForm},
      comment_report::{CommentReport, CommentReportForm},
      community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
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
      .name("timmy_crv".into())
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
      .name("sara_crv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_sara = Person::create(pool, &new_person_2).await.unwrap();

    // Add a third person, since new ppl can only report something once.
    let new_person_3 = PersonInsertForm::builder()
      .name("jessica_crv".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_jessica = Person::create(pool, &new_person_3).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test community crv".to_string())
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

    let comment_form = CommentInsertForm::builder()
      .content("A test comment 32".into())
      .creator_id(inserted_timmy.id)
      .post_id(inserted_post.id)
      .build();

    let inserted_comment = Comment::create(pool, &comment_form, None).await.unwrap();

    // sara reports
    let sara_report_form = CommentReportForm {
      creator_id: inserted_sara.id,
      comment_id: inserted_comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from sara".into(),
    };

    let inserted_sara_report = CommentReport::report(pool, &sara_report_form)
      .await
      .unwrap();

    // jessica reports
    let jessica_report_form = CommentReportForm {
      creator_id: inserted_jessica.id,
      comment_id: inserted_comment.id,
      original_comment_text: "this was it at time of creation".into(),
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = CommentReport::report(pool, &jessica_report_form)
      .await
      .unwrap();

    let agg = CommentAggregates::read(pool, inserted_comment.id)
      .await
      .unwrap();

    let read_jessica_report_view =
      CommentReportView::read(pool, inserted_jessica_report.id, inserted_timmy.id)
        .await
        .unwrap();
    let expected_jessica_report_view = CommentReportView {
      comment_report: inserted_jessica_report.clone(),
      comment: inserted_comment.clone(),
      post: inserted_post,
      community: Community {
        id: inserted_community.id,
        name: inserted_community.name,
        icon: None,
        removed: false,
        deleted: false,
        nsfw: false,
        actor_id: inserted_community.actor_id.clone(),
        local: true,
        title: inserted_community.title,
        description: None,
        updated: None,
        banner: None,
        hidden: false,
        posting_restricted_to_mods: false,
        published: inserted_community.published,
        private_key: inserted_community.private_key,
        public_key: inserted_community.public_key,
        last_refreshed_at: inserted_community.last_refreshed_at,
        followers_url: inserted_community.followers_url,
        inbox_url: inserted_community.inbox_url,
        shared_inbox_url: inserted_community.shared_inbox_url,
        moderators_url: inserted_community.moderators_url,
        featured_url: inserted_community.featured_url,
        instance_id: inserted_instance.id,
      },
      creator: Person {
        id: inserted_jessica.id,
        name: inserted_jessica.name,
        display_name: None,
        published: inserted_jessica.published,
        avatar: None,
        actor_id: inserted_jessica.actor_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_jessica.inbox_url.clone(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
        instance_id: inserted_instance.id,
        private_key: inserted_jessica.private_key,
        public_key: inserted_jessica.public_key,
        last_refreshed_at: inserted_jessica.last_refreshed_at,
      },
      comment_creator: Person {
        id: inserted_timmy.id,
        name: inserted_timmy.name.clone(),
        display_name: None,
        published: inserted_timmy.published,
        avatar: None,
        actor_id: inserted_timmy.actor_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        bot_account: false,
        bio: None,
        banner: None,
        updated: None,
        inbox_url: inserted_timmy.inbox_url.clone(),
        shared_inbox_url: None,
        matrix_user_id: None,
        ban_expires: None,
        instance_id: inserted_instance.id,
        private_key: inserted_timmy.private_key.clone(),
        public_key: inserted_timmy.public_key.clone(),
        last_refreshed_at: inserted_timmy.last_refreshed_at,
      },
      creator_banned_from_community: false,
      counts: CommentAggregates {
        comment_id: inserted_comment.id,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        published: agg.published,
        child_count: 0,
        hot_rank: 0.1728,
        controversy_rank: 0.0,
      },
      my_vote: None,
      resolver: None,
    };

    assert_eq!(read_jessica_report_view, expected_jessica_report_view);

    let mut expected_sara_report_view = expected_jessica_report_view.clone();
    expected_sara_report_view.comment_report = inserted_sara_report;
    expected_sara_report_view.creator = Person {
      id: inserted_sara.id,
      name: inserted_sara.name,
      display_name: None,
      published: inserted_sara.published,
      avatar: None,
      actor_id: inserted_sara.actor_id.clone(),
      local: true,
      banned: false,
      deleted: false,
      bot_account: false,
      bio: None,
      banner: None,
      updated: None,
      inbox_url: inserted_sara.inbox_url.clone(),
      shared_inbox_url: None,
      matrix_user_id: None,
      ban_expires: None,
      instance_id: inserted_instance.id,
      private_key: inserted_sara.private_key,
      public_key: inserted_sara.public_key,
      last_refreshed_at: inserted_sara.last_refreshed_at,
    };

    // Do a batch read of timmys reports
    let reports = CommentReportQuery::default()
      .list(pool, &timmy_view)
      .await
      .unwrap();

    assert_eq!(
      reports,
      [
        expected_jessica_report_view.clone(),
        expected_sara_report_view.clone()
      ]
    );

    // Make sure the counts are correct
    let report_count = CommentReportView::get_report_count(pool, inserted_timmy.id, false, None)
      .await
      .unwrap();
    assert_eq!(2, report_count);

    // Try to resolve the report
    CommentReport::resolve(pool, inserted_jessica_report.id, inserted_timmy.id)
      .await
      .unwrap();
    let read_jessica_report_view_after_resolve =
      CommentReportView::read(pool, inserted_jessica_report.id, inserted_timmy.id)
        .await
        .unwrap();

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
    expected_jessica_report_view_after_resolve.resolver = Some(Person {
      id: inserted_timmy.id,
      name: inserted_timmy.name.clone(),
      display_name: None,
      published: inserted_timmy.published,
      avatar: None,
      actor_id: inserted_timmy.actor_id.clone(),
      local: true,
      banned: false,
      deleted: false,
      bot_account: false,
      bio: None,
      banner: None,
      updated: None,
      inbox_url: inserted_timmy.inbox_url.clone(),
      private_key: inserted_timmy.private_key.clone(),
      public_key: inserted_timmy.public_key.clone(),
      last_refreshed_at: inserted_timmy.last_refreshed_at,
      shared_inbox_url: None,
      matrix_user_id: None,
      ban_expires: None,
      instance_id: inserted_instance.id,
    });

    assert_eq!(
      read_jessica_report_view_after_resolve,
      expected_jessica_report_view_after_resolve
    );

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = CommentReportQuery {
      unresolved_only: (true),
      ..Default::default()
    }
    .list(pool, &timmy_view)
    .await
    .unwrap();
    assert_eq!(reports_after_resolve[0], expected_sara_report_view);
    assert_eq!(reports_after_resolve.len(), 1);

    // Make sure the counts are correct
    let report_count_after_resolved =
      CommentReportView::get_report_count(pool, inserted_timmy.id, false, None)
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
