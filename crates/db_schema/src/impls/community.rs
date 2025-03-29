use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  newtypes::{CommunityId, DbUrl, PersonId},
  schema::{community, community_actions, instance, post},
  source::{
    actor_language::CommunityLanguage,
    community::{
      Community,
      CommunityActions,
      CommunityBlockForm,
      CommunityFollowerForm,
      CommunityFollowerState,
      CommunityInsertForm,
      CommunityModeratorForm,
      CommunityPersonBanForm,
      CommunityUpdateForm,
    },
    post::Post,
  },
  traits::{ApubActor, Bannable, Blockable, Crud, Followable, Joinable},
  utils::{
    functions::{coalesce, coalesce_2_nullable, lower, random_smallint},
    get_conn,
    uplete,
    DbPool,
  },
  CommunityVisibility,
  ListingType,
};
use chrono::{DateTime, Utc};
use diesel::{
  dsl::{exists, insert_into, not},
  expression::SelectableHelper,
  result::Error,
  select,
  update,
  BoolExpressionMethods,
  ExpressionMethods,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

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

impl Joinable for CommunityActions {
  type Form = CommunityModeratorForm;
  async fn join(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_actions::table)
      .values(form)
      .on_conflict((
        community_actions::person_id,
        community_actions::community_id,
      ))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityModeratorAlreadyExists)
  }

  async fn leave(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((form.person_id, form.community_id)))
      .set_null(community_actions::became_moderator)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityModeratorAlreadyExists)
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

  pub async fn update_federated_followers(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
    new_subscribers: i32,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let new_subscribers: i64 = new_subscribers.into();
    diesel::update(community::table.find(for_community_id))
      .set(community::dsl::subscribers.eq(new_subscribers))
      .get_result(conn)
      .await
  }
}

impl CommunityActions {
  pub async fn delete_mods_for_community(
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

  pub async fn leave_mod_team_for_all_communities(
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

  pub async fn approve_follower(
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

impl Bannable for CommunityActions {
  type Form = CommunityPersonBanForm;
  async fn ban(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
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
      .with_lemmy_type(LemmyErrorType::CommunityUserAlreadyBanned)
  }

  async fn unban(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((form.person_id, form.community_id)))
      .set_null(community_actions::received_ban)
      .set_null(community_actions::ban_expires)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityUserAlreadyBanned)
  }
}

impl Followable for CommunityActions {
  type Form = CommunityFollowerForm;
  type IdType = CommunityId;

  async fn follow(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
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
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)
  }
  async fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .find((person_id, community_id))
      .filter(community_actions::follow_state.is_not_null());
    diesel::update(find_action)
      .set(community_actions::follow_state.eq(Some(CommunityFollowerState::Accepted)))
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)
  }

  async fn unfollow(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    community_id: Self::IdType,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((person_id, community_id)))
      .set_null(community_actions::followed)
      .set_null(community_actions::follow_state)
      .set_null(community_actions::follow_approver_id)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)
  }
}

impl Blockable for CommunityActions {
  type Form = CommunityBlockForm;
  type ObjectIdType = CommunityId;
  type ObjectType = Community;

  async fn block(pool: &mut DbPool<'_>, form: &Self::Form) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_actions::table)
      .values(form)
      .on_conflict((
        community_actions::person_id,
        community_actions::community_id,
      ))
      .do_update()
      .set(form)
      .returning(Self::as_select())
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CommunityBlockAlreadyExists)
  }
  async fn unblock(
    pool: &mut DbPool<'_>,
    community_block_form: &Self::Form,
  ) -> LemmyResult<uplete::Count> {
    let conn = &mut get_conn(pool).await?;
    uplete::new(community_actions::table.find((
      community_block_form.person_id,
      community_block_form.community_id,
    )))
    .set_null(community_actions::blocked)
    .get_result(conn)
    .await
    .with_lemmy_type(LemmyErrorType::CommunityBlockAlreadyExists)
  }

  async fn read_block(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
    community_id: Self::ObjectIdType,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let find_action = community_actions::table
      .find((person_id, community_id))
      .filter(community_actions::blocked.is_not_null());

    select(not(exists(find_action)))
      .get_result::<bool>(conn)
      .await?
      .then_some(())
      .ok_or(LemmyErrorType::CommunityIsBlocked.into())
  }

  async fn read_blocks_for_person(
    pool: &mut DbPool<'_>,
    person_id: PersonId,
  ) -> Result<Vec<Self::ObjectType>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_actions::table
      .filter(community_actions::blocked.is_not_null())
      .inner_join(community::table)
      .select(community::all_columns)
      .filter(community_actions::person_id.eq(person_id))
      .filter(community::deleted.eq(false))
      .filter(community::removed.eq(false))
      .order_by(community_actions::blocked)
      .load::<Community>(conn)
      .await
  }
}

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
      comment::{Comment, CommentInsertForm},
      community::{
        Community,
        CommunityActions,
        CommunityFollowerForm,
        CommunityFollowerState,
        CommunityInsertForm,
        CommunityModeratorForm,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      local_user::LocalUser,
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm},
    },
    traits::{Bannable, Crud, Followable, Joinable},
    utils::{build_db_pool_for_tests, uplete, RANK_DEFAULT},
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
      posting_restricted_to_mods: false,
      instance_id: inserted_instance.id,
      visibility: CommunityVisibility::Public,
      random_number: inserted_community.random_number,
      subscribers: 1,
      posts: 0,
      comments: 0,
      users_active_day: 0,
      users_active_week: 0,
      users_active_month: 0,
      users_active_half_year: 0,
      hot_rank: RANK_DEFAULT,
      subscribers_local: 1,
      report_count: 0,
      unresolved_report_count: 0,
      interactions_month: 0,
      local_removed: false,
    };

    let community_follower_form = CommunityFollowerForm::new(
      inserted_community.id,
      inserted_bobby.id,
      CommunityFollowerState::Accepted,
    );

    let inserted_community_follower =
      CommunityActions::follow(pool, &community_follower_form).await?;

    assert_eq!(
      Some(CommunityFollowerState::Accepted),
      inserted_community_follower.follow_state
    );

    let bobby_moderator_form =
      CommunityModeratorForm::new(inserted_community.id, inserted_bobby.id);

    let inserted_bobby_moderator = CommunityActions::join(pool, &bobby_moderator_form).await?;
    assert!(inserted_bobby_moderator.became_moderator.is_some());

    let artemis_moderator_form =
      CommunityModeratorForm::new(inserted_community.id, inserted_artemis.id);

    let _inserted_artemis_moderator = CommunityActions::join(pool, &artemis_moderator_form).await?;

    let moderator_person_ids = vec![inserted_bobby.id, inserted_artemis.id];

    // Make sure bobby is marked as a higher mod than artemis, and vice versa
    let bobby_higher_check = CommunityActions::is_higher_mod_check(
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
    let artemis_higher_check = CommunityActions::is_higher_mod_check(
      pool,
      inserted_community.id,
      inserted_artemis.id,
      moderator_person_ids,
    )
    .await;
    assert!(artemis_higher_check.is_err());

    let community_person_ban_form =
      CommunityPersonBanForm::new(inserted_community.id, inserted_bobby.id);

    let inserted_community_person_ban =
      CommunityActions::ban(pool, &community_person_ban_form).await?;

    assert!(inserted_community_person_ban.received_ban.is_some());
    assert!(inserted_community_person_ban.ban_expires.is_none());
    let read_community = Community::read(pool, inserted_community.id).await?;

    let update_community_form = CommunityUpdateForm {
      title: Some("nada".to_owned()),
      ..Default::default()
    };
    let updated_community =
      Community::update(pool, inserted_community.id, &update_community_form).await?;

    let ignored_community = CommunityActions::unfollow(
      pool,
      community_follower_form.person_id,
      community_follower_form.community_id,
    )
    .await?;
    let left_community = CommunityActions::leave(pool, &bobby_moderator_form).await?;
    let unban = CommunityActions::unban(pool, &community_person_ban_form).await?;
    let num_deleted = Community::delete(pool, inserted_community.id).await?;
    Person::delete(pool, inserted_bobby.id).await?;
    Person::delete(pool, inserted_artemis.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_community, read_community);
    assert_eq!(expected_community, updated_community);
    assert_eq!(uplete::Count::only_updated(1), ignored_community);
    assert_eq!(uplete::Count::only_updated(1), left_community);
    assert_eq!(uplete::Count::only_deleted(1), unban);
    // assert_eq!(2, loaded_count);
    assert_eq!(1, num_deleted);

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn test_aggregates() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let new_person = PersonInsertForm::test_form(inserted_instance.id, "thommy_community_agg");

    let inserted_person = Person::create(pool, &new_person).await?;

    let another_person = PersonInsertForm::test_form(inserted_instance.id, "jerry_community_agg");

    let another_inserted_person = Person::create(pool, &another_person).await?;

    let new_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_community_agg".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let inserted_community = Community::create(pool, &new_community).await?;

    let another_community = CommunityInsertForm::new(
      inserted_instance.id,
      "TIL_community_agg_2".into(),
      "nada".to_owned(),
      "pubkey".to_string(),
    );
    let another_inserted_community = Community::create(pool, &another_community).await?;

    let first_person_follow = CommunityFollowerForm::new(
      inserted_community.id,
      inserted_person.id,
      CommunityFollowerState::Accepted,
    );

    CommunityActions::follow(pool, &first_person_follow).await?;

    let second_person_follow = CommunityFollowerForm::new(
      inserted_community.id,
      another_inserted_person.id,
      CommunityFollowerState::Accepted,
    );

    CommunityActions::follow(pool, &second_person_follow).await?;

    let another_community_follow = CommunityFollowerForm::new(
      another_inserted_community.id,
      inserted_person.id,
      CommunityFollowerState::Accepted,
    );

    CommunityActions::follow(pool, &another_community_follow).await?;

    let new_post = PostInsertForm::new(
      "A test post".into(),
      inserted_person.id,
      inserted_community.id,
    );
    let inserted_post = Post::create(pool, &new_post).await?;

    let comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let inserted_comment = Comment::create(pool, &comment_form, None).await?;

    let child_comment_form = CommentInsertForm::new(
      inserted_person.id,
      inserted_post.id,
      "A test comment".into(),
    );
    let _inserted_child_comment =
      Comment::create(pool, &child_comment_form, Some(&inserted_comment.path)).await?;

    let community_aggregates_before_delete = Community::read(pool, inserted_community.id).await?;

    assert_eq!(2, community_aggregates_before_delete.subscribers);
    assert_eq!(2, community_aggregates_before_delete.subscribers_local);
    assert_eq!(1, community_aggregates_before_delete.posts);
    assert_eq!(2, community_aggregates_before_delete.comments);

    // Test the other community
    let another_community_aggs = Community::read(pool, another_inserted_community.id).await?;
    assert_eq!(1, another_community_aggs.subscribers);
    assert_eq!(1, another_community_aggs.subscribers_local);
    assert_eq!(0, another_community_aggs.posts);
    assert_eq!(0, another_community_aggs.comments);

    // Unfollow test
    CommunityActions::unfollow(
      pool,
      second_person_follow.person_id,
      second_person_follow.community_id,
    )
    .await?;
    let after_unfollow = Community::read(pool, inserted_community.id).await?;
    assert_eq!(1, after_unfollow.subscribers);
    assert_eq!(1, after_unfollow.subscribers_local);

    // Follow again just for the later tests
    CommunityActions::follow(pool, &second_person_follow).await?;
    let after_follow_again = Community::read(pool, inserted_community.id).await?;
    assert_eq!(2, after_follow_again.subscribers);
    assert_eq!(2, after_follow_again.subscribers_local);

    // Remove a parent post (the comment count should also be 0)
    Post::delete(pool, inserted_post.id).await?;
    let after_parent_post_delete = Community::read(pool, inserted_community.id).await?;
    assert_eq!(0, after_parent_post_delete.posts);
    assert_eq!(0, after_parent_post_delete.comments);

    // Remove the 2nd person
    Person::delete(pool, another_inserted_person.id).await?;
    let after_person_delete = Community::read(pool, inserted_community.id).await?;
    assert_eq!(1, after_person_delete.subscribers);
    assert_eq!(1, after_person_delete.subscribers_local);

    // This should delete all the associated rows, and fire triggers
    let person_num_deleted = Person::delete(pool, inserted_person.id).await?;
    assert_eq!(1, person_num_deleted);

    // Delete the community
    let community_num_deleted = Community::delete(pool, inserted_community.id).await?;
    assert_eq!(1, community_num_deleted);

    let another_community_num_deleted =
      Community::delete(pool, another_inserted_community.id).await?;
    assert_eq!(1, another_community_num_deleted);

    // Should be none found, since the creator was deleted
    let after_delete = Community::read(pool, inserted_community.id).await;
    assert!(after_delete.is_err());

    Ok(())
  }
}
