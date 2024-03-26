use crate::structs::{CommunityModeratorView, CommunityView, PersonView};
use diesel::{
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  NullableExpressionMethods,
  PgTextExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::{CommunityId, PersonId},
  schema::{
    community,
    community_aggregates,
    community_block,
    community_follower,
    community_person_ban,
    instance_block,
    local_user,
  },
  source::{community::CommunityFollower, local_user::LocalUser, site::Site},
  utils::{fuzzy_search, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
  CommunityVisibility,
  ListingType,
  SortType,
};

fn queries<'a>() -> Queries<
  impl ReadFn<'a, CommunityView, (CommunityId, Option<PersonId>, bool)>,
  impl ListFn<'a, CommunityView, (CommunityQuery<'a>, &'a Site)>,
> {
  let all_joins = |query: community::BoxedQuery<'a, Pg>, my_person_id: Option<PersonId>| {
    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    query
      .inner_join(community_aggregates::table)
      .left_join(
        community_follower::table.on(
          community::id
            .eq(community_follower::community_id)
            .and(community_follower::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        instance_block::table.on(
          community::instance_id
            .eq(instance_block::instance_id)
            .and(instance_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        community_block::table.on(
          community::id
            .eq(community_block::community_id)
            .and(community_block::person_id.eq(person_id_join)),
        ),
      )
      .left_join(
        community_person_ban::table.on(
          community::id
            .eq(community_person_ban::community_id)
            .and(community_person_ban::person_id.eq(person_id_join)),
        ),
      )
  };

  let selection = (
    community::all_columns,
    CommunityFollower::select_subscribed_type(),
    community_block::community_id.nullable().is_not_null(),
    community_aggregates::all_columns,
    community_person_ban::person_id.nullable().is_not_null(),
  );

  let not_removed_or_deleted = community::removed
    .eq(false)
    .and(community::deleted.eq(false));

  let read = move |mut conn: DbConn<'a>,
                   (community_id, my_person_id, is_mod_or_admin): (
    CommunityId,
    Option<PersonId>,
    bool,
  )| async move {
    let mut query = all_joins(
      community::table.find(community_id).into_boxed(),
      my_person_id,
    )
    .select(selection);

    // Hide deleted and removed for non-admins or mods
    if !is_mod_or_admin {
      query = query.filter(not_removed_or_deleted);
    }

    // Hide local only communities from unauthenticated users
    if my_person_id.is_none() {
      query = query.filter(community::visibility.eq(CommunityVisibility::Public));
    }

    query.first::<CommunityView>(&mut conn).await
  };

  let list = move |mut conn: DbConn<'a>, (options, site): (CommunityQuery<'a>, &'a Site)| async move {
    use SortType::*;

    let my_person_id = options.local_user.map(|l| l.person_id);

    // The left join below will return None in this case
    let person_id_join = my_person_id.unwrap_or(PersonId(-1));

    let mut query = all_joins(community::table.into_boxed(), my_person_id)
      .left_join(local_user::table.on(local_user::person_id.eq(person_id_join)))
      .select(selection);

    if let Some(search_term) = options.search_term {
      let searcher = fuzzy_search(&search_term);
      query = query
        .filter(community::name.ilike(searcher.clone()))
        .or_filter(community::title.ilike(searcher))
    }

    // Hide deleted and removed for non-admins or mods
    if !options.is_mod_or_admin {
      query = query.filter(not_removed_or_deleted).filter(
        community::hidden
          .eq(false)
          .or(community_follower::person_id.eq(person_id_join)),
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
    };

    if let Some(listing_type) = options.listing_type {
      query = match listing_type {
        ListingType::Subscribed => query.filter(community_follower::pending.is_not_null()), // TODO could be this: and(community_follower::person_id.eq(person_id_join)),
        ListingType::Local => query.filter(community::local.eq(true)),
        _ => query,
      };
    }

    // Don't show blocked communities and communities on blocked instances. nsfw communities are
    // also hidden (based on profile setting)
    if options.local_user.is_some() {
      query = query.filter(instance_block::person_id.is_null());
      query = query.filter(community_block::person_id.is_null());
      query = query.filter(community::nsfw.eq(false).or(local_user::show_nsfw.eq(true)));
    } else {
      // No person in request, only show nsfw communities if show_nsfw is passed into request or if
      // site has content warning.
      let has_content_warning = site.content_warning.is_some();
      if !options.show_nsfw && !has_content_warning {
        query = query.filter(community::nsfw.eq(false));
      }
      // Hide local only communities from unauthenticated users
      query = query.filter(community::visibility.eq(CommunityVisibility::Public));
    }

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
    my_person_id: Option<PersonId>,
    is_mod_or_admin: bool,
  ) -> Result<Self, Error> {
    queries()
      .read(pool, (community_id, my_person_id, is_mod_or_admin))
      .await
  }

  pub async fn is_mod_or_admin(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    community_id: CommunityId,
  ) -> Result<bool, Error> {
    let is_mod =
      CommunityModeratorView::is_community_moderator(pool, community_id, person_id).await?;
    if is_mod {
      Ok(true)
    } else {
      let is_admin = PersonView::read(pool, person_id).await?.is_admin;
      Ok(is_admin)
    }
  }

  /// Checks if a person is an admin, or moderator of any community.
  pub async fn is_mod_of_any_or_admin(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<bool, Error> {
    let is_mod_of_any =
      CommunityModeratorView::is_community_moderator_of_any(pool, person_id).await?;
    if is_mod_of_any {
      return Ok(true);
    }

    let is_admin = PersonView::read(pool, person_id).await?.is_admin;
    Ok(is_admin)
  }
}

#[derive(Default)]
pub struct CommunityQuery<'a> {
  pub listing_type: Option<ListingType>,
  pub sort: Option<SortType>,
  pub local_user: Option<&'a LocalUser>,
  pub search_term: Option<String>,
  pub is_mod_or_admin: bool,
  pub show_nsfw: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl<'a> CommunityQuery<'a> {
  pub async fn list(self, site: &Site, pool: &mut DbPool<'_>) -> Result<Vec<CommunityView>, Error> {
    queries().list(pool, (self, site)).await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::indexing_slicing)]
mod tests {

  use crate::{community_view::CommunityQuery, structs::CommunityView};
  use lemmy_db_schema::{
    source::{
      community::{Community, CommunityInsertForm, CommunityUpdateForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      site::Site,
    },
    traits::Crud,
    utils::{build_db_pool_for_tests, DbPool},
    CommunityVisibility,
  };
  use serial_test::serial;
  use url::Url;

  struct Data {
    inserted_instance: Instance,
    local_user: LocalUser,
    inserted_community: Community,
    site: Site,
  }

  async fn init_data(pool: &mut DbPool<'_>) -> Data {
    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let person_name = "tegan".to_string();

    let new_person = PersonInsertForm::builder()
      .name(person_name.clone())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted(String::new())
      .build();
    let local_user = LocalUser::create(pool, &local_user_form, vec![])
      .await
      .unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test_community_3".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let url = Url::parse("http://example.com").unwrap();
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

    Data {
      inserted_instance,
      local_user,
      inserted_community,
      site,
    }
  }

  async fn cleanup(data: Data, pool: &mut DbPool<'_>) {
    Community::delete(pool, data.inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, data.local_user.person_id)
      .await
      .unwrap();
    Instance::delete(pool, data.inserted_instance.id)
      .await
      .unwrap();
  }

  #[tokio::test]
  #[serial]
  async fn local_only_community() {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();
    let data = init_data(pool).await;

    Community::update(
      pool,
      data.inserted_community.id,
      &CommunityUpdateForm {
        visibility: Some(CommunityVisibility::LocalOnly),
        ..Default::default()
      },
    )
    .await
    .unwrap();

    let unauthenticated_query = CommunityQuery {
      ..Default::default()
    }
    .list(&data.site, pool)
    .await
    .unwrap();
    assert_eq!(0, unauthenticated_query.len());

    let authenticated_query = CommunityQuery {
      local_user: Some(&data.local_user),
      ..Default::default()
    }
    .list(&data.site, pool)
    .await
    .unwrap();
    assert_eq!(1, authenticated_query.len());

    let unauthenticated_community =
      CommunityView::read(pool, data.inserted_community.id, None, false).await;
    assert!(unauthenticated_community.is_err());

    let authenticated_community = CommunityView::read(
      pool,
      data.inserted_community.id,
      Some(data.local_user.person_id),
      false,
    )
    .await;
    assert!(authenticated_community.is_ok());

    cleanup(data, pool).await;
  }
}
