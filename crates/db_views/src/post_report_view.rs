use crate::structs::PostReportView;
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  Selectable,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aggregates::structs::PostAggregatesNotInPost,
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
  source::{
    community::CommunityWithoutId,
    person::{Person, PersonWithoutId},
    post::PostWithoutId,
    post_report::PostReport,
  },
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};

type PostReportViewTuple = (
  PostReport,
  PostWithoutId,
  CommunityWithoutId,
  PersonWithoutId,
  PersonWithoutId,
  bool,
  Option<i16>,
  PostAggregatesNotInPost,
  Option<PersonWithoutId>,
);

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PostReportView, (PostReportId, PersonId)>,
  impl ListFn<'a, PostReportView, (PostReportQuery, &'a Person)>,
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
        PostWithoutId::as_select(),
        CommunityWithoutId::as_select(),
        PersonWithoutId::as_select(),
        aliases::person1.fields(<PersonWithoutId as Selectable<Pg>>::construct_selection()),
        community_person_ban::id.nullable().is_not_null(),
        post_like::score.nullable(),
        PostAggregatesNotInPost::as_select(),
        aliases::person2
          .fields(<PersonWithoutId as Selectable<Pg>>::construct_selection().nullable()),
      ))
  };

  let read = move |mut conn: DbConn<'a>, (report_id, my_person_id): (PostReportId, PersonId)| async move {
    all_joins(
      post_report::table.find(report_id).into_boxed(),
      my_person_id,
    )
    .first::<PostReportViewTuple>(&mut conn)
    .await
  };

  let list = move |mut conn: DbConn<'a>, (options, my_person): (PostReportQuery, &'a Person)| async move {
    let mut query = all_joins(post_report::table.into_boxed(), my_person.id);

    if let Some(community_id) = options.community_id {
      query = query.filter(post::community_id.eq(community_id));
    }

    if options.unresolved_only.unwrap_or(false) {
      query = query.filter(post_report::resolved.eq(false));
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query
      .order_by(post_report::published.desc())
      .limit(limit)
      .offset(offset);

    // If its not an admin, get only the ones you mod
    if !my_person.admin {
      query
        .inner_join(
          community_moderator::table.on(
            community_moderator::community_id
              .eq(post::community_id)
              .and(community_moderator::person_id.eq(my_person.id)),
          ),
        )
        .load::<PostReportViewTuple>(&mut conn)
        .await
    } else {
      query.load::<PostReportViewTuple>(&mut conn).await
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
  pub unresolved_only: Option<bool>,
}

impl PostReportQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    my_person: &Person,
  ) -> Result<Vec<PostReportView>, Error> {
    queries().list(pool, (self, my_person)).await
  }
}

impl JoinView for PostReportView {
  type JoinTuple = PostReportViewTuple;
  fn from_tuple(
    (
      post_report,
      post,
      community,
      creator,
      post_creator,
      creator_banned_from_community,
      my_vote,
      counts,
      resolver,
    ): Self::JoinTuple,
  ) -> Self {
    let post = post.into_full(post_report.post_id);
    Self {
      resolver: Option::zip(resolver, post_report.resolver_id)
        .map(|(resolver, id)| resolver.into_full(id)),
      counts: counts.into_full(&post),
      my_vote,
      creator_banned_from_community,
      post_creator: post_creator.into_full(post.creator_id),
      creator: creator.into_full(post_report.creator_id),
      community: community.into_full(post.community_id),
      post,
      post_report,
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::post_report_view::{PostReportQuery, PostReportView};
  use lemmy_db_schema::{
    aggregates::structs::PostAggregates,
    source::{
      community::{Community, CommunityInsertForm, CommunityModerator, CommunityModeratorForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
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

    let inserted_sara_report = PostReport::report(pool, &sara_report_form).await.unwrap();

    // jessica reports
    let jessica_report_form = PostReportForm {
      creator_id: inserted_jessica.id,
      post_id: inserted_post.id,
      original_post_name: "Orig post".into(),
      original_post_url: None,
      original_post_body: None,
      reason: "from jessica".into(),
    };

    let inserted_jessica_report = PostReport::report(pool, &jessica_report_form)
      .await
      .unwrap();

    let agg = PostAggregates::read(pool, inserted_post.id).await.unwrap();

    let read_jessica_report_view =
      PostReportView::read(pool, inserted_jessica_report.id, inserted_timmy.id)
        .await
        .unwrap();
    let expected_jessica_report_view = PostReportView {
      post_report: inserted_jessica_report.clone(),
      post: inserted_post.clone(),
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
        instance_id: inserted_instance.id,
        private_key: inserted_community.private_key.clone(),
        public_key: inserted_community.public_key.clone(),
        last_refreshed_at: inserted_community.last_refreshed_at,
        followers_url: inserted_community.followers_url.clone(),
        inbox_url: inserted_community.inbox_url.clone(),
        shared_inbox_url: inserted_community.shared_inbox_url.clone(),
        moderators_url: inserted_community.moderators_url.clone(),
        featured_url: inserted_community.featured_url.clone(),
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
        admin: false,
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
      post_creator: Person {
        id: inserted_timmy.id,
        name: inserted_timmy.name.clone(),
        display_name: None,
        published: inserted_timmy.published,
        avatar: None,
        actor_id: inserted_timmy.actor_id.clone(),
        local: true,
        banned: false,
        deleted: false,
        admin: false,
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
      my_vote: None,
      counts: PostAggregates {
        id: agg.id,
        post_id: inserted_post.id,
        comments: 0,
        score: 0,
        upvotes: 0,
        downvotes: 0,
        published: agg.published,
        newest_comment_time_necro: inserted_post.published,
        newest_comment_time: inserted_post.published,
        featured_community: false,
        featured_local: false,
        hot_rank: 1728,
        hot_rank_active: 1728,
        controversy_rank: 0.0,
        community_id: inserted_post.community_id,
        creator_id: inserted_post.creator_id,
      },
      resolver: None,
    };

    assert_eq!(read_jessica_report_view, expected_jessica_report_view);

    let mut expected_sara_report_view = expected_jessica_report_view.clone();
    expected_sara_report_view.post_report = inserted_sara_report;
    expected_sara_report_view.my_vote = None;
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
      admin: false,
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
    let reports = PostReportQuery::default()
      .list(pool, &inserted_timmy)
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
    let report_count = PostReportView::get_report_count(pool, inserted_timmy.id, false, None)
      .await
      .unwrap();
    assert_eq!(2, report_count);

    // Try to resolve the report
    PostReport::resolve(pool, inserted_jessica_report.id, inserted_timmy.id)
      .await
      .unwrap();
    let read_jessica_report_view_after_resolve =
      PostReportView::read(pool, inserted_jessica_report.id, inserted_timmy.id)
        .await
        .unwrap();

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
      admin: false,
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
    });

    assert_eq!(
      read_jessica_report_view_after_resolve,
      expected_jessica_report_view_after_resolve
    );

    // Do a batch read of timmys reports
    // It should only show saras, which is unresolved
    let reports_after_resolve = PostReportQuery {
      unresolved_only: (Some(true)),
      ..Default::default()
    }
    .list(pool, &inserted_timmy)
    .await
    .unwrap();
    assert_eq!(reports_after_resolve[0], expected_sara_report_view);

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
