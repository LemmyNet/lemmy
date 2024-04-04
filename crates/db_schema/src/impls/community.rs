use crate::{
  diesel::DecoratableTarget,
  newtypes::{CommunityId, DbUrl, PersonId},
  schema::{community, community_follower, instance},
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
    use crate::schema::community_moderator::dsl::community_moderator;
    let conn = &mut get_conn(pool).await?;
    insert_into(community_moderator)
      .values(community_moderator_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn leave(
    pool: &mut DbPool<'_>,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_moderator::dsl::community_moderator;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_moderator.find((
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

  /// Get the community which has a given moderators or featured url, also return the collection type
  pub async fn get_by_collection_url(
    pool: &mut DbPool<'_>,
    url: &DbUrl,
  ) -> Result<(Community, CollectionType), Error> {
    use crate::schema::community::dsl::{featured_url, moderators_url};
    use CollectionType::*;
    let conn = &mut get_conn(pool).await?;
    let res = community::table
      .filter(moderators_url.eq(url))
      .first::<Self>(conn)
      .await;
    if let Ok(c) = res {
      return Ok((c, Moderators));
    }
    let res = community::table
      .filter(featured_url.eq(url))
      .first::<Self>(conn)
      .await;
    if let Ok(c) = res {
      return Ok((c, Featured));
    }
    Err(diesel::NotFound)
  }

  pub async fn set_featured_posts(
    community_id: CommunityId,
    posts: Vec<Post>,
    pool: &mut DbPool<'_>,
  ) -> Result<(), Error> {
    use crate::schema::post;
    let conn = &mut get_conn(pool).await?;
    for p in &posts {
      debug_assert!(p.community_id == community_id);
    }
    // Mark the given posts as featured and all other posts as not featured.
    let post_ids = posts.iter().map(|p| p.id);
    update(post::table)
      .filter(post::dsl::community_id.eq(community_id))
      // This filter is just for performance
      .filter(post::dsl::featured_community.or(post::dsl::id.eq_any(post_ids.clone())))
      .set(post::dsl::featured_community.eq(post::dsl::id.eq_any(post_ids)))
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
    use crate::schema::community_moderator::dsl::{community_id, community_moderator};
    let conn = &mut get_conn(pool).await?;

    diesel::delete(community_moderator.filter(community_id.eq(for_community_id)))
      .execute(conn)
      .await
  }

  pub async fn leave_all_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<usize, Error> {
    use crate::schema::community_moderator::dsl::{community_moderator, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_moderator.filter(person_id.eq(for_person_id)))
      .execute(conn)
      .await
  }

  pub async fn get_person_moderated_communities(
    pool: &mut DbPool<'_>,
    for_person_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error> {
    use crate::schema::community_moderator::dsl::{community_id, community_moderator, person_id};
    let conn = &mut get_conn(pool).await?;
    community_moderator
      .filter(person_id.eq(for_person_id))
      .select(community_id)
      .load::<CommunityId>(conn)
      .await
  }
}

#[async_trait]
impl Bannable for CommunityPersonBan {
  type Form = CommunityPersonBanForm;
  async fn ban(
    pool: &mut DbPool<'_>,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_person_ban::dsl::{community_id, community_person_ban, person_id};
    let conn = &mut get_conn(pool).await?;
    insert_into(community_person_ban)
      .values(community_person_ban_form)
      .on_conflict((community_id, person_id))
      .do_update()
      .set(community_person_ban_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn unban(
    pool: &mut DbPool<'_>,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_person_ban::dsl::community_person_ban;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_person_ban.find((
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
    use crate::schema::community_follower::dsl::{community_follower, community_id};
    let conn = &mut get_conn(pool).await?;
    select(exists(
      community_follower.filter(community_id.eq(remote_community_id)),
    ))
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
    use crate::schema::community_follower::dsl::{community_follower, community_id, person_id};
    let conn = &mut get_conn(pool).await?;
    insert_into(community_follower)
      .values(form)
      .on_conflict((community_id, person_id))
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
    use crate::schema::community_follower::dsl::{community_follower, pending};
    let conn = &mut get_conn(pool).await?;
    diesel::update(community_follower.find((person_id, community_id)))
      .set(pending.eq(false))
      .get_result::<Self>(conn)
      .await
  }
  async fn unfollow(pool: &mut DbPool<'_>, form: &CommunityFollowerForm) -> Result<usize, Error> {
    use crate::schema::community_follower::dsl::community_follower;
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community_follower.find((form.person_id, form.community_id)))
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
    Ok(
      community::table
        .filter(community::actor_id.eq(object_id))
        .first::<Community>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  async fn read_from_name(
    pool: &mut DbPool<'_>,
    community_name: &str,
    include_deleted: bool,
  ) -> Result<Community, Error> {
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
    q.first::<Self>(conn).await
  }

  async fn read_from_name_and_domain(
    pool: &mut DbPool<'_>,
    community_name: &str,
    for_domain: &str,
  ) -> Result<Community, Error> {
    let conn = &mut get_conn(pool).await?;
    community::table
      .inner_join(instance::table)
      .filter(lower(community::name).eq(community_name.to_lowercase()))
      .filter(lower(instance::domain).eq(for_domain.to_lowercase()))
      .select(community::all_columns)
      .first::<Self>(conn)
      .await
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
  use pretty_assertions::assert_eq;
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
      .name("bobbee".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("TIL".into())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

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
      person_id: inserted_person.id,
      pending: false,
    };

    let inserted_community_follower = CommunityFollower::follow(pool, &community_follower_form)
      .await
      .unwrap();

    let expected_community_follower = CommunityFollower {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
      published: inserted_community_follower.published,
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
    };

    let inserted_community_moderator = CommunityModerator::join(pool, &community_moderator_form)
      .await
      .unwrap();

    let expected_community_moderator = CommunityModerator {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      published: inserted_community_moderator.published,
    };

    let community_person_ban_form = CommunityPersonBanForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      expires: None,
    };

    let inserted_community_person_ban = CommunityPersonBan::ban(pool, &community_person_ban_form)
      .await
      .unwrap();

    let expected_community_person_ban = CommunityPersonBan {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      published: inserted_community_person_ban.published,
      expires: None,
    };

    let read_community = Community::read(pool, inserted_community.id).await.unwrap();

    let update_community_form = CommunityUpdateForm {
      title: Some("nada".to_owned()),
      ..Default::default()
    };
    let updated_community = Community::update(pool, inserted_community.id, &update_community_form)
      .await
      .unwrap();

    let ignored_community = CommunityFollower::unfollow(pool, &community_follower_form)
      .await
      .unwrap();
    let left_community = CommunityModerator::leave(pool, &community_moderator_form)
      .await
      .unwrap();
    let unban = CommunityPersonBan::unban(pool, &community_person_ban_form)
      .await
      .unwrap();
    let num_deleted = Community::delete(pool, inserted_community.id)
      .await
      .unwrap();
    Person::delete(pool, inserted_person.id).await.unwrap();
    Instance::delete(pool, inserted_instance.id).await.unwrap();

    assert_eq!(expected_community, read_community);
    assert_eq!(expected_community, inserted_community);
    assert_eq!(expected_community, updated_community);
    assert_eq!(expected_community_follower, inserted_community_follower);
    assert_eq!(expected_community_moderator, inserted_community_moderator);
    assert_eq!(expected_community_person_ban, inserted_community_person_ban);
    assert_eq!(1, ignored_community);
    assert_eq!(1, left_community);
    assert_eq!(1, unban);
    // assert_eq!(2, loaded_count);
    assert_eq!(1, num_deleted);
  }
}
