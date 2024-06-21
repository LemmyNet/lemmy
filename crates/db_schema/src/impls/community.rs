use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  newtypes::{CommunityId, DbUrl, PersonId},
  schema::{
    community,
    community_follower,
    community_moderator,
    community_person_ban,
    instance,
    post,
  },
  source::{
    actor_language::CommunityLanguage,
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
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
    functions::{coalesce, lower},
    get_conn,
    DbPool,
  },
  SubscribedType,
};
use chrono::{DateTime, Utc};
use diesel::{
  deserialize,
  dsl,
  dsl::{exists, insert_into},
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
    insert_into(community_moderator::table)
      .values(community_moderator_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn leave(
    pool: &mut DbPool<'_>,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_moderator::table.find((
      community_moderator_form.person_id,
      community_moderator_form.community_id,
    )))
    .execute(conn)
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
    let is_new_community = match &form.actor_id {
      Some(id) => Community::read_from_apub_id(pool, id).await?.is_none(),
      None => true,
    };
    let conn = &mut get_conn(pool).await?;

    // Can't do separate insert/update commands because InsertForm/UpdateForm aren't convertible
    let community_ = insert_into(community::table)
      .values(form)
      .on_conflict(community::actor_id)
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
  ) -> Result<Option<(Community, CollectionType)>, Error> {
    let conn = &mut get_conn(pool).await?;
    let res = community::table
      .filter(community::moderators_url.eq(url))
      .first(conn)
      .await
      .optional()?;

    if let Some(c) = res {
      Ok(Some((c, CollectionType::Moderators)))
    } else {
      let res = community::table
        .filter(community::featured_url.eq(url))
        .first(conn)
        .await
        .optional()?;
      if let Some(c) = res {
        Ok(Some((c, CollectionType::Featured)))
      } else {
        Ok(None)
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
}

impl CommunityModerator {
  pub async fn delete_for_community(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;

    diesel::delete(
      community_moderator::table.filter(community_moderator::community_id.eq(for_community_id)),
    )
    .execute(conn)
    .await
  }

  pub async fn leave_all_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      community_moderator::table.filter(community_moderator::person_id.eq(for_person_id)),
    )
    .execute(conn)
    .await
  }

  pub async fn get_person_moderated_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error> {
    let conn = &mut get_conn(pool).await?;
    community_moderator::table
      .filter(community_moderator::person_id.eq(for_person_id))
      .select(community_moderator::community_id)
      .load::<CommunityId>(conn)
      .await
  }

  /// Checks to make sure the acting moderator is higher than the target moderator
  pub async fn is_higher_mod_check(
    pool: &mut DbPool<'_>,
    for_community_id: CommunityId,
    mod_person_id: PersonId,
    target_person_ids: &[PersonId],
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;

    // Build the list of persons
    let mut persons = target_person_ids.to_owned();
    persons.push(mod_person_id);
    persons.dedup();

    let res = community_moderator::table
      .filter(community_moderator::community_id.eq(for_community_id))
      .filter(community_moderator::person_id.eq_any(persons))
      .order_by(community_moderator::published)
      // This does a limit 1 select first
      .first::<CommunityModerator>(conn)
      .await?;

    // If the first result sorted by published is the acting mod
    if res.person_id == mod_person_id {
      Ok(true)
    } else {
      Err(diesel::result::Error::NotFound)
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
    insert_into(community_person_ban::table)
      .values(community_person_ban_form)
      .on_conflict((
        community_person_ban::community_id,
        community_person_ban::person_id,
      ))
      .do_update()
      .set(community_person_ban_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn unban(
    pool: &mut DbPool<'_>,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_person_ban::table.find((
      community_person_ban_form.person_id,
      community_person_ban_form.community_id,
    )))
    .execute(conn)
    .await
  }
}

impl CommunityFollower {
  pub fn to_subscribed_type(follower: &Option<Self>) -> SubscribedType {
    match follower {
      Some(f) => {
        if f.pending {
          SubscribedType::Pending
        } else {
          SubscribedType::Subscribed
        }
      }
      // If the row doesn't exist, the person isn't a follower.
      None => SubscribedType::NotSubscribed,
    }
  }

  pub fn select_subscribed_type() -> dsl::Nullable<community_follower::pending> {
    community_follower::pending.nullable()
  }

  /// Check if a remote instance has any followers on local instance. For this it is enough to check
  /// if any follow relation is stored. Dont use this for local community.
  pub async fn has_local_followers(
    pool: &mut DbPool<'_>,
    remote_community_id: CommunityId,
  ) -> Result<bool, Error> {
    let conn = &mut get_conn(pool).await?;
    select(exists(community_follower::table.filter(
      community_follower::community_id.eq(remote_community_id),
    )))
    .get_result(conn)
    .await
  }
}

impl Queryable<sql_types::Nullable<sql_types::Bool>, Pg> for SubscribedType {
  type Row = Option<bool>;
  fn build(row: Self::Row) -> deserialize::Result<Self> {
    Ok(match row {
      Some(true) => SubscribedType::Pending,
      Some(false) => SubscribedType::Subscribed,
      None => SubscribedType::NotSubscribed,
    })
  }
}

#[async_trait]
impl Followable for CommunityFollower {
  type Form = CommunityFollowerForm;
  async fn follow(pool: &mut DbPool<'_>, form: &CommunityFollowerForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_follower::table)
      .values(form)
      .on_conflict((
        community_follower::community_id,
        community_follower::person_id,
      ))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
  async fn follow_accepted(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    person_id: PersonId,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(community_follower::table.find((person_id, community_id)))
      .set(community_follower::pending.eq(false))
      .get_result::<Self>(conn)
      .await
  }

  async fn unfollow(pool: &mut DbPool<'_>, form: &CommunityFollowerForm) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_follower::table.find((form.person_id, form.community_id)))
      .execute(conn)
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
      .filter(community::actor_id.eq(object_id))
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
      q = q
        .filter(community::deleted.eq(false))
        .filter(community::removed.eq(false));
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
#[allow(clippy::indexing_slicing)]
mod tests {
  use crate::{
    source::{
      community::{
        Community,
        CommunityFollower,
        CommunityFollowerForm,
        CommunityInsertForm,
        CommunityModerator,
        CommunityModeratorForm,
        CommunityPersonBan,
        CommunityPersonBanForm,
        CommunityUpdateForm,
      },
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    traits::{Bannable, Crud, Followable, Joinable},
    utils::build_db_pool_for_tests,
    CommunityVisibility,
  };
  use lemmy_utils::{error::LemmyResult, LemmyErrorType};
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let bobby_person = PersonInsertForm::test_form(inserted_instance.id, "bobby");
    let inserted_bobby = Person::create(pool, &bobby_person).await?;

    let artemis_person = PersonInsertForm::test_form(inserted_instance.id, "artemis");
    let inserted_artemis = Person::create(pool, &artemis_person).await?;

    let new_community = CommunityInsertForm::builder()
      .name("TIL".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await?;

    let expected_community = Community {
      id: inserted_community.id,
      name: "TIL".into(),
      title: "nada".to_owned(),
      description: None,
      nsfw: false,
      removed: false,
      deleted: false,
      published: inserted_community.published,
      updated: None,
      actor_id: inserted_community.actor_id.clone(),
      local: true,
      private_key: None,
      public_key: "pubkey".to_owned(),
      last_refreshed_at: inserted_community.published,
      icon: None,
      banner: None,
      followers_url: inserted_community.followers_url.clone(),
      inbox_url: inserted_community.inbox_url.clone(),
      shared_inbox_url: None,
      moderators_url: None,
      featured_url: None,
      hidden: false,
      posting_restricted_to_mods: false,
      instance_id: inserted_instance.id,
      visibility: CommunityVisibility::Public,
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      pending: false,
    };

    let inserted_community_follower =
      CommunityFollower::follow(pool, &community_follower_form).await?;

    let expected_community_follower = CommunityFollower {
      community_id: inserted_community.id,
      person_id: inserted_bobby.id,
      pending: false,
      published: inserted_community_follower.published,
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
      &moderator_person_ids,
    )
    .await?;
    assert!(bobby_higher_check);

    // This should throw an error, since artemis was added later
    let artemis_higher_check = CommunityModerator::is_higher_mod_check(
      pool,
      inserted_community.id,
      inserted_artemis.id,
      &moderator_person_ids,
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

    let read_community = Community::read(pool, inserted_community.id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?;

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
    assert_eq!(1, ignored_community);
    assert_eq!(1, left_community);
    assert_eq!(1, unban);
    // assert_eq!(2, loaded_count);
    assert_eq!(1, num_deleted);

    Ok(())
  }
}
