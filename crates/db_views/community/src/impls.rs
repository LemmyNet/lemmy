use crate::{CommunityView, MultiCommunityView};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use i_love_jesus::asc_if;
use lemmy_db_schema::{
  CommunitySortType,
  MultiCommunityListingType,
  MultiCommunitySortType,
  impls::local_user::LocalUserOptionHelper,
  newtypes::{CommunityId, MultiCommunityId},
  source::{
    community::{Community, community_keys as key},
    local_user::LocalUser,
    multi_community::{MultiCommunity, multi_community_keys as mkey},
    site::Site,
  },
  utils::{
    limit_fetch,
    queries::filters::{
      filter_is_subscribed,
      filter_not_unlisted_or_is_subscribed,
      filter_suggested_communities,
    },
  },
};
use lemmy_db_schema_file::{
  PersonId,
  enums::ListingType,
  joins::{
    my_community_actions_join,
    my_instance_communities_actions_join,
    my_local_user_admin_join,
    my_multi_community_follower_join,
  },
  schema::{
    community,
    community_actions,
    instance_actions,
    multi_community,
    multi_community_entry,
    multi_community_follow,
    person,
  },
};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  pagination::{
    CursorData,
    PagedResponse,
    PaginationCursor,
    PaginationCursorConversion,
    paginate_response,
  },
  traits::Crud,
  utils::{LowerKey, now, seconds_to_pg_interval},
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl CommunityView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(person_id: Option<PersonId>) -> _ {
    let community_actions_join: my_community_actions_join = my_community_actions_join(person_id);
    let instance_actions_community_join: my_instance_communities_actions_join =
      my_instance_communities_actions_join(person_id);
    let my_local_user_admin_join: my_local_user_admin_join = my_local_user_admin_join(person_id);

    community::table
      .left_join(community_actions_join)
      .left_join(instance_actions_community_join)
      .left_join(my_local_user_admin_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    my_local_user: Option<&'_ LocalUser>,
    is_mod_or_admin: bool,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let mut query = Self::joins(my_local_user.person_id())
      .filter(community::id.eq(community_id))
      .select(Self::as_select())
      .into_boxed();

    // Hide deleted and removed for non-admins or mods
    if !is_mod_or_admin {
      query = query
        .filter(Community::hide_removed_and_deleted())
        .filter(filter_not_unlisted_or_is_subscribed());
    }

    query = my_local_user.visible_communities_only(query);

    query
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorConversion for CommunityView {
  type PaginatedType = Community;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.community.id.0)
  }

  async fn from_cursor(
    data: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    Community::read(pool, CommunityId(data.id()?)).await
  }
}

#[derive(Default)]
pub struct CommunityQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<CommunitySortType>,
  pub time_range_seconds: Option<i32>,
  pub local_user: Option<&'a LocalUser>,
  pub show_nsfw: Option<bool>,
  pub multi_community_id: Option<MultiCommunityId>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
}

impl CommunityQuery<'_> {
  pub async fn list(
    self,
    site: &Site,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<PagedResponse<CommunityView>> {
    use lemmy_db_schema::CommunitySortType::*;
    let o = self;
    let limit = limit_fetch(o.limit, None)?;

    let mut query = CommunityView::joins(o.local_user.person_id())
      .select(CommunityView::as_select())
      .limit(limit)
      .into_boxed();

    // Hide deleted and removed for non-admins
    let is_admin = o.local_user.map(|l| l.admin).unwrap_or_default();
    if !is_admin {
      query = query
        .filter(Community::hide_removed_and_deleted())
        .filter(filter_not_unlisted_or_is_subscribed());
    }

    if let Some(listing_type) = o.listing_type {
      query = match listing_type {
        ListingType::All => query.filter(filter_not_unlisted_or_is_subscribed()),
        ListingType::Subscribed => query.filter(filter_is_subscribed()),
        ListingType::Local => query
          .filter(community::local.eq(true))
          .filter(filter_not_unlisted_or_is_subscribed()),
        ListingType::ModeratorView => {
          query.filter(community_actions::became_moderator_at.is_not_null())
        }
        ListingType::Suggested => query.filter(filter_suggested_communities()),
      };
    }

    // Don't show blocked communities and communities on blocked instances. nsfw communities are
    // also hidden (based on profile setting)
    query = query.filter(instance_actions::blocked_communities_at.is_null());
    query = query.filter(community_actions::blocked_at.is_null());
    if !(o.local_user.show_nsfw(site) || o.show_nsfw.unwrap_or_default()) {
      query = query.filter(community::nsfw.eq(false));
    }

    query = o.local_user.visible_communities_only(query);

    if let Some(multi_community_id) = o.multi_community_id {
      let communities = multi_community_entry::table
        .filter(multi_community_entry::multi_community_id.eq(multi_community_id))
        .select(multi_community_entry::community_id);
      query = query.filter(community::id.eq_any(communities))
    }

    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query = query
        .filter(community::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)));
    }

    // Only sort by ascending for Old or NameAsc sorts.
    let sort = o.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == Old || sort == NameAsc);

    let mut pq = CommunityView::paginate(query, &o.page_cursor, sort_direction, pool, None).await?;

    pq = match sort {
      Hot => pq.then_order_by(key::hot_rank),
      Comments => pq.then_order_by(key::comments),
      Posts => pq.then_order_by(key::posts),
      New => pq.then_order_by(key::published_at),
      Old => pq.then_order_by(key::published_at),
      Subscribers => pq.then_order_by(key::subscribers),
      SubscribersLocal => pq.then_order_by(key::subscribers_local),
      ActiveSixMonths => pq.then_order_by(key::users_active_half_year),
      ActiveMonthly => pq.then_order_by(key::users_active_month),
      ActiveWeekly => pq.then_order_by(key::users_active_week),
      ActiveDaily => pq.then_order_by(key::users_active_day),
      NameAsc => pq.then_order_by(LowerKey(key::name)),
      NameDesc => pq.then_order_by(LowerKey(key::name)),
    };

    // finally use unique id as tie breaker
    pq = pq.then_order_by(key::id);

    let conn = &mut get_conn(pool).await?;
    let res = pq
      .load::<CommunityView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;
    paginate_response(res, limit, o.page_cursor)
  }
}

impl MultiCommunityView {
  #[diesel::dsl::auto_type(no_type_alias)]
  fn joins(person_id: Option<PersonId>) -> _ {
    let my_multi_community_follower_join: my_multi_community_follower_join =
      my_multi_community_follower_join(person_id);

    multi_community::table
      .inner_join(person::table)
      .left_join(my_multi_community_follower_join)
  }

  pub async fn read(
    pool: &mut DbPool<'_>,
    id: MultiCommunityId,
    my_person_id: Option<PersonId>,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;

    Self::joins(my_person_id)
      .filter(multi_community::id.eq(id))
      .select(Self::as_select())
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl PaginationCursorConversion for MultiCommunityView {
  type PaginatedType = MultiCommunity;
  fn to_cursor(&self) -> CursorData {
    CursorData::new_id(self.multi.id.0)
  }

  async fn from_cursor(
    data: CursorData,
    pool: &mut DbPool<'_>,
  ) -> LemmyResult<Self::PaginatedType> {
    MultiCommunity::read(pool, MultiCommunityId(data.id()?)).await
  }
}

#[derive(Default)]
pub struct MultiCommunityQuery {
  pub listing_type: Option<MultiCommunityListingType>,
  pub sort: Option<MultiCommunitySortType>,
  pub time_range_seconds: Option<i32>,
  pub my_person_id: Option<PersonId>,
  pub creator_id: Option<PersonId>,
  pub page_cursor: Option<PaginationCursor>,
  pub limit: Option<i64>,
  pub no_limit: Option<bool>,
}

impl MultiCommunityQuery {
  pub async fn list(self, pool: &mut DbPool<'_>) -> LemmyResult<PagedResponse<MultiCommunityView>> {
    use lemmy_db_schema::{MultiCommunityListingType::*, MultiCommunitySortType::*};
    let o = self;

    let mut query = MultiCommunityView::joins(o.my_person_id)
      .select(MultiCommunityView::as_select())
      .into_boxed();

    let limit = limit_fetch(o.limit, o.no_limit)?;
    query = query.limit(limit);

    if let Some(listing_type) = o.listing_type {
      query = match listing_type {
        All => query,
        Subscribed => {
          if let Some(my_person_id) = o.my_person_id {
            query.filter(multi_community_follow::person_id.eq(my_person_id))
          } else {
            query
          }
        }
        Local => query.filter(multi_community::local),
      };
    }

    if let Some(creator_id) = o.creator_id {
      query = query.filter(multi_community::creator_id.eq(creator_id));
    }

    // Filter by the time range
    if let Some(time_range_seconds) = o.time_range_seconds {
      query = query.filter(
        multi_community::published_at.gt(now() - seconds_to_pg_interval(time_range_seconds)),
      );
    }

    // Only sort by ascending for Old or NameAsc sorts.
    let sort = o.sort.unwrap_or_default();
    let sort_direction = asc_if(sort == Old || sort == NameAsc);

    let mut pq =
      MultiCommunityView::paginate(query, &o.page_cursor, sort_direction, pool, None).await?;

    pq = match sort {
      New => pq.then_order_by(mkey::published_at),
      Old => pq.then_order_by(mkey::published_at),
      Communities => pq.then_order_by(mkey::communities),
      Subscribers => pq.then_order_by(mkey::subscribers),
      SubscribersLocal => pq.then_order_by(mkey::subscribers_local),
      NameAsc => pq.then_order_by(LowerKey(mkey::name)),
      NameDesc => pq.then_order_by(LowerKey(mkey::name)),
    };

    // finally use unique id as tie breaker
    pq = pq.then_order_by(mkey::id);

    let conn = &mut get_conn(pool).await?;
    let res = pq
      .load::<MultiCommunityView>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)?;

    paginate_response(res, limit, o.page_cursor)
  }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{
    CommunityView,
    impls::{CommunityQuery, MultiCommunityListingType, MultiCommunityQuery},
  };
  use lemmy_db_schema::{
    CommunitySortType,
    source::{
      community::{
        Community,
        CommunityActions,
        CommunityFollowerForm,
        CommunityInsertForm,
        CommunityModeratorForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      multi_community::{MultiCommunity, MultiCommunityFollowForm, MultiCommunityInsertForm},
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::Followable,
  };
  use lemmy_db_schema_file::enums::{CommunityFollowerState, CommunityVisibility};
  use lemmy_diesel_utils::{
    connection::{DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::error::{LemmyErrorType, LemmyResult};
  use serial_test::serial;
  use std::collections::HashSet;
  use url::Url;

  struct Data {
    instance: Instance,
    local_user: LocalUser,
    communities: [Community; 3],
    site: Site,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> LemmyResult<Data> {
    let instance = Instance::read_or_create(pool, "my_domain.tld").await?;

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
      published_at: Default::default(),
      updated_at: None,
      icon: None,
      banner: None,
      summary: None,
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
  async fn follow_state() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;
    let community = &data.communities[0];

    let unauthenticated = CommunityView::read(pool, community.id, None, false).await?;
    assert!(unauthenticated.community_actions.is_none());

    let authenticated =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert!(authenticated.community_actions.is_none());

    let form = CommunityFollowerForm::new(
      community.id,
      data.local_user.person_id,
      CommunityFollowerState::Pending,
    );
    CommunityActions::follow(pool, &form).await?;

    let with_pending_follow =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert!(
      with_pending_follow
        .community_actions
        .is_some_and(|x| x.follow_state == Some(CommunityFollowerState::Pending))
    );

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
    let form = CommunityFollowerForm::new(
      community.id,
      data.local_user.person_id,
      CommunityFollowerState::ApprovalRequired,
    );
    CommunityActions::follow(pool, &form).await?;

    let with_approval_required_follow =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert!(
      with_approval_required_follow
        .community_actions
        .is_some_and(|x| x.follow_state == Some(CommunityFollowerState::ApprovalRequired))
    );

    let form = CommunityFollowerForm::new(
      community.id,
      data.local_user.person_id,
      CommunityFollowerState::Accepted,
    );
    CommunityActions::follow(pool, &form).await?;
    let with_accepted_follow =
      CommunityView::read(pool, community.id, Some(&data.local_user), false).await?;
    assert!(
      with_accepted_follow
        .community_actions
        .is_some_and(|x| x.follow_state == Some(CommunityFollowerState::Accepted))
    );

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
        visibility: Some(CommunityVisibility::LocalOnlyPrivate),
        ..Default::default()
      },
    )
    .await?;

    let unauthenticated_query = CommunityQuery {
      sort: Some(CommunitySortType::New),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?;
    assert_eq!(data.communities.len() - 1, unauthenticated_query.len());

    let authenticated_query = CommunityQuery {
      local_user: Some(&data.local_user),
      sort: Some(CommunitySortType::New),
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
      sort: Some(CommunitySortType::New),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .iter()
    .for_each(|c| assert!(!c.can_mod));

    let person_id = data.local_user.person_id;

    // Now join the mod team of test community 1 and 2
    let mod_form_1 = CommunityModeratorForm::new(data.communities[0].id, person_id);
    CommunityActions::join(pool, &mod_form_1).await?;

    let mod_form_2 = CommunityModeratorForm::new(data.communities[1].id, person_id);
    CommunityActions::join(pool, &mod_form_2).await?;

    let mod_query = CommunityQuery {
      local_user: Some(&data.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await?
    .iter()
    .map(|c| (c.community.name.clone(), c.can_mod))
    .collect::<HashSet<_>>();

    let expected_communities = HashSet::from([
      ("test_community_3".to_owned(), false),
      ("test_community_2".to_owned(), true),
      ("test_community_1".to_owned(), true),
    ]);
    assert_eq!(expected_communities, mod_query);

    cleanup(data, pool).await
  }

  #[tokio::test]
  #[serial]
  async fn test_multi_community_list() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let data = init_data(pool).await?;

    let tom_form = PersonInsertForm::test_form(data.instance.id, "tom");
    let tom = Person::create(pool, &tom_form).await?;

    let multi_1_form = MultiCommunityInsertForm::new(
      data.local_user.person_id,
      data.instance.id,
      "multi2".to_string(),
      String::new(),
    );
    let multi = MultiCommunity::create(pool, &multi_1_form).await?;

    let multi_2_form =
      MultiCommunityInsertForm::new(tom.id, tom.instance_id, "multi2".to_string(), String::new());
    let multi2 = MultiCommunity::create(pool, &multi_2_form).await?;

    // list all multis
    let list_all = MultiCommunityQuery::default()
      .list(pool)
      .await?
      .iter()
      .map(|m| m.multi.id)
      .collect::<HashSet<_>>();

    assert_eq!(list_all, HashSet::from([multi.id, multi2.id]));

    // list multis by owner
    let list_owner = MultiCommunityQuery {
      creator_id: Some(data.local_user.person_id),
      my_person_id: Some(data.local_user.person_id),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(list_owner.len(), 1);
    assert_eq!(list_owner[0].multi.id, multi.id);
    assert_eq!(list_owner[0].follow_state, None);

    // Tegan follows multi2
    let follow_form = MultiCommunityFollowForm {
      multi_community_id: multi2.id,
      person_id: data.local_user.person_id,
      follow_state: CommunityFollowerState::Accepted,
    };
    MultiCommunity::follow(pool, &follow_form).await?;

    // list multis followed by user, followed_only
    let list_followed = MultiCommunityQuery {
      my_person_id: Some(data.local_user.person_id),
      listing_type: Some(MultiCommunityListingType::Subscribed),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(list_followed.len(), 1);
    assert_eq!(list_followed[0].multi.id, multi2.id);
    assert_eq!(list_followed[0].owner.id, tom.id);
    assert_eq!(
      list_followed[0].follow_state,
      Some(CommunityFollowerState::Accepted)
    );

    // Unfollow, and make sure its removed
    MultiCommunity::unfollow(pool, data.local_user.person_id, multi2.id).await?;
    let list_followed = MultiCommunityQuery {
      my_person_id: Some(data.local_user.person_id),
      listing_type: Some(MultiCommunityListingType::Subscribed),
      ..Default::default()
    }
    .list(pool)
    .await?;
    assert_eq!(list_followed.len(), 0);

    cleanup(data, pool).await?;

    Ok(())
  }
}
