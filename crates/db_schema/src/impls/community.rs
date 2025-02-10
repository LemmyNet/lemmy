use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  newtypes::{CommunityId, DbUrl, PersonId},
  schema::{community, community_actions, instance, post},
  source::{
    actor_language::CommunityLanguage,
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityFollowerState,
      CommunityInsertForm,
      CommunityModerator,
      CommunityModeratorForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
      CommunityUpdateForm,
    },
    post::Post,
  },
  traits::{ApubActor, Bannable, Crud, Followable, Joinable},
  utils::{
    functions::{coalesce, coalesce_2_nullable, lower, random_smallint},
    get_conn,
    now,
    uplete,
    DbPool,
  },
  CommunityVisibility,
  ListingType,
  SubscribedType,
};
use chrono::{DateTime, Utc};
use diesel::{
  deserialize,
  dsl::{exists, insert_into, not},
  expression::SelectableHelper,
  pg::Pg,
  result::Error,
  select,
  sql_types,
  update,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
  Queryable,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

#[async_trait]
impl Crud for Community {
  type InsertForm = CommunityInsertForm;
  type UpdateForm = CommunityUpdateForm;
  type IdType = CommunityId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;

    let community_ = insert_into(community::table)
      .values(form)
      .get_result::<Self>(conn)
      .await?;

    // Initialize languages for new community
    CommunityLanguage::update(pool, vec![], community_.id).await?;

    Ok(community_)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(community::table.find(community_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Joinable for CommunityModerator {
  type Form = CommunityModeratorForm;
  async fn join(
    pool: &mut DbPool<'_>,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let community_moderator_form = (
      community_moderator_form,
      community_actions::became_moderator.eq(now().nullable()),
    );
    insert_into(community_actions::table)
      .values(community_moderator_form)
      .on_conflict((
        community_actions::person_id,
        community_actions::community_id,
      ))
      .do_update()
      .set(community_moderator_form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }

  async fn leave(
    pool: &mut DbPool<'_>,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((
      community_moderator_form.person_id,
      community_moderator_form.community_id,
    )))
    .set_null(community_actions::became_moderator)
    .get_result(conn)
    .await
  }
}

pub enum CollectionType {
  Moderators,
  Featured,
}

impl Community {
  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: DateTime<Utc>,
    form: &CommunityInsertForm,
  ) -> Result<Self, Error> {
    let is_new_community = match &form.ap_id {
      Some(id) => Community::read_from_apub_id(pool, id).await?.is_none(),
      None => true,
    };
    let conn = &mut get_conn(pool).await?;

    // Can't do separate insert/update commands because InsertForm/UpdateForm aren't convertible
    let community_ = insert_into(community::table)
      .values(form)
      .on_conflict(community::ap_id)
      .filter_target(coalesce(community::updated, community::published).lt(timestamp))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await?;

    // Initialize languages for new community
    if is_new_community {
      CommunityLanguage::update(pool, vec![], community_.id).await?;
    }

    Ok(community_)
  }

  /// Get the community which has a given moderators or featured url, also return the collection
  /// type
  pub async fn get_by_collection_url(
    pool: &mut DbPool<'_>,
    url: &DbUrl,
  ) -> LemmyResult<(Community, CollectionType)> {
    let conn = &mut get_conn(pool).await?;
    let res = community::table
      .filter(community::moderators_url.eq(url))
      .first(conn)
      .await;

    if let Ok(c) = res {
      Ok((c, CollectionType::Moderators))
    } else {
      let res = community::table
        .filter(community::featured_url.eq(url))
        .first(conn)
        .await;
      if let Ok(c) = res {
        Ok((c, CollectionType::Featured))
      } else {
        Err(LemmyErrorType::NotFound.into())
      }
    }
  }

  pub async fn set_featured_posts(
    community_id: CommunityId,
    posts: Vec<Post>,
    pool: &mut DbPool<'_>,
  ) -> Result<(), Error> {
    let conn = &mut get_conn(pool).await?;
    for p in &posts {
      debug_assert!(p.community_id == community_id);
    }
    // Mark the given posts as featured and all other posts as not featured.
    let post_ids = posts.iter().map(|p| p.id);
    update(post::table)
      .filter(post::community_id.eq(community_id))
      // This filter is just for performance
      .filter(post::featured_community.or(post::id.eq_any(post_ids.clone())))
      .set(post::featured_community.eq(post::id.eq_any(post_ids)))
      .execute(conn)
      .await?;
    Ok(())
  }

  pub async fn get_random_community_id(
    pool: &mut DbPool<'_>,
    type_: &Option<ListingType>,
    show_nsfw: Option<bool>,
  ) -> Result<CommunityId, Error> {
    let conn = &mut get_conn(pool).await?;

    // This is based on the random page selection algorithm in MediaWiki. It assigns a random number
    // X to each item. To pick a random one, it generates a random number Y and gets the item with
    // the lowest X value where X >= Y.
    //
    // https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/specials/SpecialRandomPage.php;763c5f084101676ab1bc52862e1ffbd24585a365
    //
    // The difference is we also regenerate the item's assigned number when the item is picked.
    // Without this, items would have permanent variations in the probability of being picked.
    // Additionally, in each group of multiple items that are assigned the same random number (a
    // more likely occurence with `smallint`), there would be only one item that ever gets
    // picked.

    let try_pick = || {
      let mut query = community::table
        .filter(not(
          community::deleted
            .or(community::removed)
            .or(community::visibility.eq(CommunityVisibility::Private)),
        ))
        .order(community::random_number.asc())
        .select(community::id)
        .into_boxed();

      if let Some(ListingType::Local) = type_ {
        query = query.filter(community::local);
      }

      if !show_nsfw.unwrap_or(false) {
        query = query.filter(not(community::nsfw));
      }

      query
    };

    diesel::update(community::table)
      .filter(
        community::id.nullable().eq(coalesce_2_nullable(
          try_pick()
            .filter(community::random_number.nullable().ge(
              // Without `select` and `single_value`, this would call `random_smallint` separately
              // for each row
              select(random_smallint()).single_value(),
            ))
            .single_value(),
          // Wrap to the beginning if the generated number is higher than all
          // `community::random_number` values, just like in the MediaWiki algorithm
          try_pick().single_value(),
        )),
      )
      .set(community::random_number.eq(random_smallint()))
      .returning(community::id)
      .get_result::<CommunityId>(conn)
      .await
  }

  #[diesel::dsl::auto_type(no_type_alias)]
  pub fn hide_removed_and_deleted() -> _ {
    community::removed
      .eq(false)
      .and(community::deleted.eq(false))
  }

  pub fn local_url(name: &str, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/c/{name}"))?.into())
  }
}

impl CommunityModerator {
  pub async fn delete_for_community(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;

    uplete::new(
      community_actions::table.filter(community_actions::community_id.eq(for_community_id)),
    )
    .set_null(community_actions::became_moderator)
    .get_result(conn)
    .await
  }

  pub async fn leave_all_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.filter(community_actions::person_id.eq(for_person_id)))
      .set_null(community_actions::became_moderator)
      .get_result(conn)
      .await
  }

  pub async fn get_person_moderated_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_actions::table
      .filter(community_actions::became_moderator.is_not_null())
      .filter(community_actions::person_id.eq(for_person_id))
      .select(community_actions::community_id)
      .load::<CommunityId>(conn)
      .await
  }

  /// Checks to make sure the acting moderator was added earlier than the target moderator
  pub async fn is_higher_mod_check(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
    mod_person_id: PersonId,
    target_person_ids: Vec<PersonId>,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;

    // Build the list of persons
    let mut persons = target_person_ids;
    persons.push(mod_person_id);
    persons.dedup();

    let res = community_actions::table
      .filter(community_actions::became_moderator.is_not_null())
      .filter(community_actions::community_id.eq(for_community_id))
      .filter(community_actions::person_id.eq_any(persons))
      .order_by(community_actions::became_moderator)
      .select(community_actions::person_id)
      // This does a limit 1 select first
      .first::<PersonId>(conn)
      .await?;

    // If the first result sorted by published is the acting mod
    if res == mod_person_id {
      Ok(())
    } else {
      Err(LemmyErrorType::NotHigherMod)?
    }
  }
}

#[async_trait]
impl Bannable for CommunityPersonBan {
  type Form = CommunityPersonBanForm;
  async fn ban(
    pool: &mut DbPool<'_>,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let community_person_ban_form = (
      community_person_ban_form,
      community_actions::received_ban.eq(now().nullable()),
    );
    insert_into(community_actions::table)
      .values(community_person_ban_form)
      .on_conflict((
        community_actions::community_id,
        community_actions::person_id,
      ))
      .do_update()
      .set(community_person_ban_form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }

  async fn unban(
    pool: &mut DbPool<'_>,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((
      community_person_ban_form.person_id,
      community_person_ban_form.community_id,
    )))
    .set_null(community_actions::received_ban)
    .set_null(community_actions::ban_expires)
    .get_result(conn)
    .await
  }
}

impl CommunityFollower {
  /// Check if a remote instance has any followers on local instance. For this it is enough to check
  /// if any follow relation is stored. Dont use this for local community.
  pub async fn check_has_local_followers(
    pool: &mut DbPool<'_>,
    remote_community_id: CommunityId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .filter(community_actions::followed.is_not_null())
      .filter(community_actions::community_id.eq(remote_community_id));
    select(exists(find_action))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(LemmyErrorType::CommunityHasNoFollowers.into())
  }

  pub async fn approve(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    follower_id: PersonId,
    approver_id: PersonId,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .find((follower_id, community_id))
      .filter(community_actions::followed.is_not_null());
    diesel::update(find_action)
      .set((
        community_actions::follow_state.eq(CommunityFollowerState::Accepted),
        community_actions::follow_approver_id.eq(approver_id),
      ))
      .execute(conn)
      .await?;
    Ok(())
  }
}

// TODO
// I'd really like to have these on the impl, but unfortunately they have to be top level,
// according to https://diesel.rs/guides/composing-applications.html
#[diesel::dsl::auto_type]
pub fn community_follower_select_subscribed_type() -> _ {
  community_actions::follow_state.nullable()
}

impl Queryable<sql_types::Nullable<crate::schema::sql_types::CommunityFollowerState>, Pg>
  for SubscribedType
{
  type Row = Option<CommunityFollowerState>;
  fn build(row: Self::Row) -> deserialize::Result<Self> {
    Ok(match row {
      Some(CommunityFollowerState::Pending) => SubscribedType::Pending,
      Some(CommunityFollowerState::Accepted) => SubscribedType::Subscribed,
      Some(CommunityFollowerState::ApprovalRequired) => SubscribedType::ApprovalRequired,
      None => SubscribedType::NotSubscribed,
    })
  }
}

#[async_trait]
impl Followable for CommunityFollower {
  type Form = CommunityFollowerForm;
  async fn follow(pool: &mut DbPool<'_>, form: &CommunityFollowerForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let form = (form, community_actions::followed.eq(now().nullable()));
    insert_into(community_actions::table)
      .values(form)
      .on_conflict((
        community_actions::community_id,
        community_actions::person_id,
      ))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .find((person_id, community_id))
      .filter(community_actions::follow_state.is_not_null());
    diesel::update(find_action)
      .set(community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted)))
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
  }
  async fn unfollow(
    pool: &mut DbPool<'_>,
    form: &CommunityFollowerForm,
  ) -> Result<uplete::Count, Error> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((form.person_id, form.community_id)))
      .set_null(community_actions::followed)
      .set_null(community_actions::follow_state)
      .set_null(community_actions::follow_approver_id)
      .get_result(conn)
      .await
  }
}

#[async_trait]
impl ApubActor for Community {
  async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: &DbUrl,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community::table
      .filter(community::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    community_name: &str,
    include_deleted: bool,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut q = community::table
      .into_boxed()
      .filter(community::local.eq(true))
      .filter(lower(community::name).eq(community_name.to_lowercase()));
    if !include_deleted {
      q = q.filter(Self::hide_removed_and_deleted())
    }
    q.first(conn).await.optional()
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    community_name: &str,
    for_domain: &str,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    community::table
      .inner_join(instance::table)
      .filter(lower(community::name).eq(community_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(community::all_columns)
      .first(conn)
      .await
      .optional()
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{
      community::{
        Community,
        CommunityFollower,
        CommunityFollowerForm,
        CommunityFollowerState,
        CommunityInsertForm,
        CommunityModerator,
        CommunityModeratorForm,
        CommunityPersonBan,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      local_user::LocalUser,
      person::{Person, PersonInsertForm},
    },
    traits::{Bannable, Crud, Followable, Joinable},
    utils::{build_db_pool_for_tests, uplete},
    CommunityVisibility,
  };
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let bobby_person = PersonInsertForm::test_form(inserted_instance.id, "bobby");
    let inserted_bobby = Person::create(pool, &bobby_person).await?;

    let artemis_person = PersonInsertForm::test_form(inserted_instance.id, "artemis");
    let inserted_artemis = Person::create(pool, &artemis_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let expected_community = Community {
      id: inserted_community.id,
      name: "TIL".into(),
      title: "nada".to_owned(),
      sidebar: None,
      description: None,
      nsfw: false,
      removed: false,
      deleted: false,
      published: inserted_community.published,
      updated: None,
      ap_id: inserted_community.ap_id.clone(),
      local: true,
      private_key: None,
      public_key: "pubkey".to_owned(),
      last_refreshed_at: inserted_community.published,
      icon: None,
      banner: None,
      followers_url: inserted_community.followers_url.clone(),
      inbox_url: inserted_community.inbox_url.clone(),
      moderators_url: None,
      featured_url: None,
      hidden: false,
      posting_restricted_to_mods: false,
      instance_id: inserted_instance.id,
      visibility: CommunityVisibility::Public,
      random_number: inserted_community.random_number,
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      state: Some(CommunityFollowerState::Accepted),
      approver_id: None,
    };

    let inserted_community_follower =
      CommunityFollower::follow(pool, &community_follower_form).await?;

    let expected_community_follower = CommunityFollower {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      state: CommunityFollowerState::Accepted,
      published: inserted_community_follower.published,
      approver_id: None,
    };

    let bobby_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
    };

    let inserted_bobby_moderator = CommunityModerator::join(pool, &bobby_moderator_form).await?;

    let artemis_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_artemis.id,
    };

    let _inserted_artemis_moderator =
      CommunityModerator::join(pool, &artemis_moderator_form).await?;

    let expected_community_moderator = CommunityModerator {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      published: inserted_bobby_moderator.published,
    };

    let moderator_person_ids = vec![inserted_bobby.id, inserted_artemis.id];

    // Make sure bobby is marked as a higher mod than artemis, and vice versa
    let bobby_higher_check = CommunityModerator::is_higher_mod_check(
      pool,
      inserted_community.id,
      inserted_bobby.id,
      moderator_person_ids.clone(),
    )
    .await;
    assert!(bobby_higher_check.is_ok());

    // Also check the other is_higher_mod_or_admin function just in case
    let bobby_higher_check_2 = LocalUser::is_higher_mod_or_admin_check(
      pool,
      inserted_community.id,
      inserted_bobby.id,
      moderator_person_ids.clone(),
    )
    .await;
    assert!(bobby_higher_check_2.is_ok());

    // This should throw an error, since artemis was added later
    let artemis_higher_check = CommunityModerator::is_higher_mod_check(
      pool,
      inserted_community.id,
      inserted_artemis.id,
      moderator_person_ids,
    )
    .await;
    assert!(artemis_higher_check.is_err());

    let community_person_ban_form = CommunityPersonBanForm {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      expires: None,
    };

    let inserted_community_person_ban =
      CommunityPersonBan::ban(pool, &community_person_ban_form).await?;

    let expected_community_person_ban = CommunityPersonBan {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      published: inserted_community_person_ban.published,
      expires: None,
    };

    let read_community = Community::read(pool, inserted_community.id).await?;

    let update_community_form = CommunityUpdateForm {
      title: Some("nada".to_owned()),
      ..Default::default()
    };
    let updated_community =
      Community::update(pool, inserted_community.id, &update_community_form).await?;

    let ignored_community = CommunityFollower::unfollow(pool, &community_follower_form).await?;
    let left_community = CommunityModerator::leave(pool, &bobby_moderator_form).await?;
    let unban = CommunityPersonBan::unban(pool, &community_person_ban_form).await?;
    let num_deleted = Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_bobby.id).await?;
    Person::delete(pool, inserted_artemis.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_community, read_community);
    assert_eq!(expected_community, inserted_community);
    assert_eq!(expected_community, updated_community);
    assert_eq!(expected_community_follower, inserted_community_follower);
    assert_eq!(expected_community_moderator, inserted_bobby_moderator);
    assert_eq!(expected_community_person_ban, inserted_community_person_ban);
    assert_eq!(uplete::Count::only_updated(1), ignored_community);
    assert_eq!(uplete::Count::only_updated(1), left_community);
    assert_eq!(uplete::Count::only_deleted(1), unban);
    // assert_eq!(2, loaded_count);
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
