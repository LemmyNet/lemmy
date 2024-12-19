use crate::structs::{CommunityModeratorView, CommunitySortType, CommunityView, PersonView};
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, PersonId},
  schema::{community, community_actions, community_aggregates, instance_actions},
  source::{
    community::{CommunityFollower, CommunityFollowerState},
    local_user::LocalUser,
    site::Site,
  },
  utils::{
    actions,
    functions::lower,
    fuzzy_search,
    limit_and_offset,
    DbConn,
    DbPool,
    ListFn,
    Queries,
    ReadFn,
  },
  ListingType,
  PostSortType,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

type QueriesReadTypes<'a> = (CommunityId, Option<&'a LocalUser>, bool);
type QueriesListTypes<'a> = (CommunityQuery<'a>, &'a Site);

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommunityView, QueriesReadTypes<'a>>,
  impl ListFn<'a, CommunityView, QueriesListTypes<'a>>,
> {
  let all_joins = |query: community::BoxedQuery<'a, Pg>, my_local_user: Option<&'a LocalUser>| {
    query
      .inner_join(community_aggregates::table)
      .left_join(actions(
        community_actions::table,
        my_local_user.person_id(),
        community::id,
      ))
      .left_join(actions(
        instance_actions::table,
        my_local_user.person_id(),
        community::instance_id,
      ))
  };

  let selection = (
    community::all_columns,
    CommunityFollower::select_subscribed_type(),
    community_actions::blocked.nullable().is_not_null(),
    community_aggregates::all_columns,
    community_actions::received_ban.nullable().is_not_null(),
  );

  let not_removed_or_deleted = community::removed
    .eq(false)
    .and(community::deleted.eq(false));

  let read = move |mut conn: DbConn<'a>,
                   (community_id, my_local_user, is_mod_or_admin): (
    CommunityId,
    Option<&'a LocalUser>,
    bool,
  )| async move {
    let mut query = all_joins(
      community::table.find(community_id).into_boxed(),
      my_local_user,
    )
    .select(selection);

    // Hide deleted and removed for non-admins or mods
    if !is_mod_or_admin {
      query = query.filter(not_removed_or_deleted);
    }

    query = my_local_user.visible_communities_only(query);

    query.first(&mut conn).await
  };

  let list = move |mut conn: DbConn<'a>, (options, site): (CommunityQuery<'a>, &'a Site)| async move {
    use CommunitySortType::*;

    let mut query = all_joins(community::table.into_boxed(), options.local_user).select(selection);

    if let Some(search_term) = options.search_term {
      let searcher = fuzzy_search(&search_term);
      let name_filter = community::name.ilike(searcher.clone());
      let title_filter = community::title.ilike(searcher.clone());
      let description_filter = community::description.ilike(searcher.clone());
      query = if options.title_only.unwrap_or_default() {
        query.filter(name_filter.or(title_filter))
      } else {
        query.filter(name_filter.or(title_filter.or(description_filter)))
      }
    }

    // Hide deleted and removed for non-admins or mods
    if !options.is_mod_or_admin {
      query = query.filter(not_removed_or_deleted).filter(
        community::hidden
          .eq(false)
          .or(community_actions::follow_state.is_not_null()),
      );
    }

    match options.sort.unwrap_or(Hot) {
      Hot | Active | Scaled => query = query.order_by(community_aggregates::hot_rank.desc()),
      NewComments | TopDay | TopTwelveHour | TopSixHour | TopHour => {
        query = query.order_by(community_aggregates::users_active_day.desc())
      }
      New => query = query.order_by(community::published.desc()),
      Old => query = query.order_by(community::published.asc()),
      // Controversial is temporary until a CommentSortType is created
      MostComments | Controversial => query = query.order_by(community_aggregates::comments.desc()),
      TopAll | TopYear | TopNineMonths => {
        query = query.order_by(community_aggregates::subscribers.desc())
      }
      TopSixMonths | TopThreeMonths => {
        query = query.order_by(community_aggregates::users_active_half_year.desc())
      }
      TopMonth => query = query.order_by(community_aggregates::users_active_month.desc()),
      TopWeek => query = query.order_by(community_aggregates::users_active_week.desc()),
      NameAsc => query = query.order_by(lower(community::name).asc()),
      NameDesc => query = query.order_by(lower(community::name).desc()),
    };

    if let Some(listing_type) = options.listing_type {
      query = match listing_type {
        ListingType::Subscribed => {
          query.filter(community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted)))
        }
        ListingType::Local => query.filter(community::local.eq(true)),
        _ => query,
      };
    }

    // Don't show blocked communities and communities on blocked instances. nsfw communities are
    // also hidden (based on profile setting)
    query = query.filter(instance_actions::blocked.is_null());
    query = query.filter(community_actions::blocked.is_null());
    if !(options.local_user.show_nsfw(site) || options.show_nsfw) {
      query = query.filter(community::nsfw.eq(false));
    }

    query = options.local_user.visible_communities_only(query);

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;
    query
      .limit(limit)
      .offset(offset)
      .load::<CommunityView>(&mut conn)
      .await
  };

  Queries::new(read, list)
}

impl CommunityView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    my_local_user: Option<&'_ LocalUser>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    queries()
      .read(pool, (community_id, my_local_user, is_mod_or_admin))
      .await
  }

  pub async fn check_is_mod_or_admin(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    community_id: CommunityId,
  ) -> LemmyResult<()> {
    let is_mod =
      CommunityModeratorView::check_is_community_moderator(pool, community_id, person_id).await;
    if is_mod.is_ok()
      || PersonView::read(pool, person_id, false)
        .await
        .is_ok_and(|t| t.is_admin)
    {
      Ok(())
    } else {
      Err(LemmyErrorType::NotAModOrAdmin)?
    }
  }

  /// Checks if a person is an admin, or moderator of any community.
  pub async fn check_is_mod_of_any_or_admin(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> LemmyResult<()> {
    let is_mod_of_any =
      CommunityModeratorView::is_community_moderator_of_any(pool, person_id).await;
    if is_mod_of_any.is_ok()
      || PersonView::read(pool, person_id, false)
        .await
        .is_ok_and(|t| t.is_admin)
    {
      Ok(())
    } else {
      Err(LemmyErrorType::NotAModOrAdmin)?
    }
  }
}

impl From<PostSortType> for CommunitySortType {
  fn from(value: PostSortType) -> Self {
    match value {
      PostSortType::Active => Self::Active,
      PostSortType::Hot => Self::Hot,
      PostSortType::New => Self::New,
      PostSortType::Old => Self::Old,
      PostSortType::TopDay => Self::TopDay,
      PostSortType::TopWeek => Self::TopWeek,
      PostSortType::TopMonth => Self::TopMonth,
      PostSortType::TopYear => Self::TopYear,
      PostSortType::TopAll => Self::TopAll,
      PostSortType::MostComments => Self::MostComments,
      PostSortType::NewComments => Self::NewComments,
      PostSortType::TopHour => Self::TopHour,
      PostSortType::TopSixHour => Self::TopSixHour,
      PostSortType::TopTwelveHour => Self::TopTwelveHour,
      PostSortType::TopThreeMonths => Self::TopThreeMonths,
      PostSortType::TopSixMonths => Self::TopSixMonths,
      PostSortType::TopNineMonths => Self::TopNineMonths,
      PostSortType::Controversial => Self::Controversial,
      PostSortType::Scaled => Self::Scaled,
    }
  }
}

#[derive(Default)]
pub struct CommunityQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommunitySortType>,
  pub local_user: Option<&'a LocalUser>,
  pub search_term: Option<String>,
  pub title_only: Option<bool>,
  pub is_mod_or_admin: bool,
  pub show_nsfw: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl CommunityQuery<'_> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> Result<Vec<CommunityView>, Error> {
    queries().list(pool, (self, site)).await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    community_view::CommunityQuery,
    structs::{CommunitySortType, CommunityView},
  };
  use lemmy_db_schema::{
    source::{
      community::{
        Community,
        CommunityFollower,
        CommunityFollowerForm,
        CommunityFollowerState,
        CommunityInsertForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::{Crud, Followable},
    utils::{build_db_pool_for_tests, DbPool},
    CommunityVisibility,
    SubscribedType,
  };
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use serial_test::serial;
  use url::Url;

  struct Data {
    instance: Instance,
    local_user: LocalUser,
    communities: [Community; 3],
    site: Site,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let person_name = "tegan".to_string();

    let new_person = PersonInsertForm::test_form(instance.id, &person_name);

    let inserted_person = Person::create(pool, &new_person).await?;

    let local_user_form = LocalUserInsertForm::test_form(inserted_person.id);
    let local_user = LocalUser::create(pool, &local_user_form, vec![]).await?;

    let communities = [
      Community::create(
        pool,
        &CommunityInsertForm::new(
          instance.id,
          "test_community_1".to_string(),
          "nada1".to_owned(),
          "pubkey".to_string(),
        ),
      )
      .await?,
      Community::create(
        pool,
        &CommunityInsertForm::new(
          instance.id,
          "test_community_2".to_string(),
          "nada2".to_owned(),
          "pubkey".to_string(),
        ),
      )
      .await?,
      Community::create(
        pool,
        &CommunityInsertForm::new(
          instance.id,
          "test_community_3".to_string(),
          "nada3".to_owned(),
          "pubkey".to_string(),
        ),
      )
      .await?,
    ];

    let url = Url::parse("http://example.com")?;
    let site = Site {
      id: Default::default(),
      name: String::new(),
      sidebar: None,
      published: Default::default(),
      updated: None,
      icon: None,
      banner: None,
      description: None,
      actor_id: url.clone().into(),
      last_refreshed_at: Default::default(),
      inbox_url: url.into(),
      private_key: None,
      public_key: String::new(),
      instance_id: Default::default(),
      content_warning: None,
    };

    Ok(Data {
      instance,
      local_user,
      communities,
      site,
    })
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    for Community { id, .. } in data.communities {
      Community::delete(pool, id).await?;
    }
    Person::delete(pool, data.local_user.person_id).await?;
    Instance::delete(pool, data.instance.id).await?;

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn subscribe_state() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    let community = &data.communities[0];

    let unauthenticated = CommunityView::read(pool, community.id, None, false).await?;
    assert_eq!(SubscribedType::NotSubscribed, unauthenticated.subscribed);

    let authenticated =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert_eq!(SubscribedType::NotSubscribed, authenticated.subscribed);

    let form = CommunityFollowerForm {
      state: Some(CommunityFollowerState::Pending),
      ..CommunityFollowerForm::new(community.id, data.local_user.person_id)
    };
    CommunityFollower::follow(pool, &form).await?;

    let with_pending_follow =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert_eq!(SubscribedType::Pending, with_pending_follow.subscribed);

    // mark community private and set follow as approval required
    Community::update(
      pool,
      community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::Private),
        ..Default::default()
      },
    )
    .await?;
    let form = CommunityFollowerForm {
      state: Some(CommunityFollowerState::ApprovalRequired),
      ..CommunityFollowerForm::new(community.id, data.local_user.person_id)
    };
    CommunityFollower::follow(pool, &form).await?;

    let with_approval_required_follow =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert_eq!(
      SubscribedType::ApprovalRequired,
      with_approval_required_follow.subscribed
    );

    let form = CommunityFollowerForm {
      state: Some(CommunityFollowerState::Accepted),
      ..CommunityFollowerForm::new(community.id, data.local_user.person_id)
    };
    CommunityFollower::follow(pool, &form).await?;
    let with_accepted_follow =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert_eq!(SubscribedType::Subscribed, with_accepted_follow.subscribed);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn local_only_community() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    Community::update(
      pool,
      data.communities[0].id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::LocalOnly),
        ..Default::default()
      },
    )
    .await?;

    let unauthenticated_query = CommunityQuery {
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(data.communities.len() - 1, unauthenticated_query.len());

    let authenticated_query = CommunityQuery {
      local_user: Some(&data.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(data.communities.len(), authenticated_query.len());

    let unauthenticated_community =
      CommunityView::read(pool, data.communities[0].id, None, false).await;
    assert!(unauthenticated_community.is_err());

    let authenticated_community =
      CommunityView::read(pool, data.communities[0].id, Some(&data.local_user), false).await;
    assert!(authenticated_community.is_ok());

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn community_sort_name() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let query = CommunityQuery {
      sort: Some(CommunitySortType::NameAsc),
      ..Default::default()
    };
    let communities = query.list(&data.site, pool).await?;
    for (i, c) in communities.iter().enumerate().skip(1) {
      let prev = communities.get(i - 1).ok_or(LemmyErrorType::NotFound)?;
      assert!(c.community.title.cmp(&prev.community.title).is_ge());
    }

    let query = CommunityQuery {
      sort: Some(CommunitySortType::NameDesc),
      ..Default::default()
    };
    let communities = query.list(&data.site, pool).await?;
    for (i, c) in communities.iter().enumerate().skip(1) {
      let prev = communities.get(i - 1).ok_or(LemmyErrorType::NotFound)?;
      assert!(c.community.title.cmp(&prev.community.title).is_le());
    }

    cleanup(data, pool).await
  }
}
