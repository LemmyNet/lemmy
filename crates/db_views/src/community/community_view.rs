use crate::structs::{CommunityModeratorView, CommunitySortType, CommunityView, PersonView};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
  SelectableHelper,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, PersonId},
  schema::{community, community_actions, community_aggregates, instance_actions, local_user},
  source::{
    community::{Community, CommunityFollowerState},
    local_user::LocalUser,
    site::Site,
  },
  utils::{functions::lower, get_conn, limit_and_offset, now, seconds_to_pg_interval, DbPool},
  ListingType,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

impl CommunityView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(person_id: Option<PersonId>) -> _ {
    let community_actions_join = community_actions::table.on(
      community_actions::community_id
        .eq(community::id)
        .and(community_actions::person_id.nullable().eq(person_id)),
    );

    let instance_actions_join = instance_actions::table.on(
      instance_actions::instance_id
        .eq(community::instance_id)
        .and(instance_actions::person_id.nullable().eq(person_id)),
    );

    let local_user_join = local_user::table.on(local_user::person_id.nullable().eq(person_id));

    community::table
      .inner_join(community_aggregates::table)
      .left_join(community_actions_join)
      .left_join(instance_actions_join)
      .left_join(local_user_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    my_local_user: Option<&'_ LocalUser>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins(my_local_user.person_id())
      .filter(community::id.eq(community_id))
      .select(Self::as_select())
      .into_boxed();

    // Hide deleted and removed for non-admins or mods
    if !is_mod_or_admin {
      query = query.filter(Community::hide_removed_and_deleted());
    }

    query = my_local_user.visible_communities_only(query);

    query.first(conn).await
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

#[derive(Default)]
pub struct CommunityQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommunitySortType>,
  pub time_range_seconds: Option<i32>,
  pub local_user: Option<&'a LocalUser>,
  pub title_only: Option<bool>,
  pub is_mod_or_admin: bool,
  pub show_nsfw: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl CommunityQuery<'_> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> Result<Vec<CommunityView>, Error> {
    use CommunitySortType::*;
    let conn = &mut get_conn(pool).await?;
    let o = self;

    let mut query = CommunityView::joins(o.local_user.person_id())
      .select(CommunityView::as_select())
      .into_boxed();

    // Hide deleted and removed for non-admins or mods
    if !o.is_mod_or_admin {
      query = query.filter(Community::hide_removed_and_deleted()).filter(
        community::hidden
          .eq(false)
          .or(community_actions::follow_state.is_not_null()),
      );
    }

    let is_subscribed = community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted));

    if let Some(listing_type) = o.listing_type {
      query = match listing_type {
        ListingType::All => query.filter(community::hidden.eq(false).or(is_subscribed)),
        ListingType::Subscribed => query.filter(is_subscribed),
        ListingType::Local => query
          .filter(community::local.eq(true))
          .filter(community::hidden.eq(false).or(is_subscribed)),
        ListingType::ModeratorView => {
          query.filter(community_actions::became_moderator.is_not_null())
        }
      };
    }

    // Don't show blocked communities and communities on blocked instances. nsfw communities are
    // also hidden (based on profile setting)
    query = query.filter(instance_actions::blocked.is_null());
    query = query.filter(community_actions::blocked.is_null());
    if !(o.local_user.show_nsfw(site) || o.show_nsfw) {
      query = query.filter(community::nsfw.eq(false));
    }

    query = o.local_user.visible_communities_only(query);

    match o.sort.unwrap_or_default() {
      Hot => query = query.order_by(community_aggregates::hot_rank.desc()),
      Comments => query = query.order_by(community_aggregates::comments.desc()),
      Posts => query = query.order_by(community_aggregates::posts.desc()),
      New => query = query.order_by(community::published.desc()),
      Old => query = query.order_by(community::published.asc()),
      Subscribers => query = query.order_by(community_aggregates::subscribers.desc()),
      SubscribersLocal => query = query.order_by(community_aggregates::subscribers_local.desc()),
      ActiveSixMonths => {
        query = query.order_by(community_aggregates::users_active_half_year.desc())
      }
      ActiveMonthly => query = query.order_by(community_aggregates::users_active_month.desc()),
      ActiveWeekly => query = query.order_by(community_aggregates::users_active_week.desc()),
      ActiveDaily => query = query.order_by(community_aggregates::users_active_day.desc()),
      NameAsc => query = query.order_by(lower(community::name).asc()),
      NameDesc => query = query.order_by(lower(community::name).desc()),
    };
    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query = query.filter(
        community_aggregates::published.gt(now() - seconds_to_pg_interval(time_range_seconds)),
      );
    }

    let (limit, offset) = limit_and_offset(o.page, o.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .load::<CommunityView>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {

  use crate::{
    community::community_view::CommunityQuery,
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
        CommunityModerator,
        CommunityModeratorForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::{Crud, Followable, Joinable},
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
      ap_id: url.clone().into(),
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

  #[tokio::test]
  #[serial]
  async fn can_mod() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    // Make sure can_mod is false for all of them.
    CommunityQuery {
      local_user: Some(&data.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .for_each(|c| assert!(!c.can_mod));

    let person_id = data.local_user.person_id;

    // Now join the mod team of test community 1 and 2
    let mod_form_1 = CommunityModeratorForm {
      community_id: data.communities[0].id,
      person_id,
    };
    CommunityModerator::join(pool, &mod_form_1).await?;

    let mod_form_2 = CommunityModeratorForm {
      community_id: data.communities[1].id,
      person_id,
    };
    CommunityModerator::join(pool, &mod_form_2).await?;

    let mod_query = CommunityQuery {
      local_user: Some(&data.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .into_iter()
    .map(|c| (c.community.name, c.can_mod))
    .collect::<Vec<_>>();

    let expected_communities = vec![
      ("test_community_1".to_owned(), true),
      ("test_community_2".to_owned(), true),
      ("test_community_3".to_owned(), false),
    ];
    assert_eq!(expected_communities, mod_query);

    cleanup(data, pool).await
  }
}
