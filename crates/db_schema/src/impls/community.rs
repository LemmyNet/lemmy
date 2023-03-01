use crate::{
  newtypes::{CommunityId, DbUrl, PersonId},
  schema::community::dsl::{actor_id, community, deleted, local, name, removed},
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityInsertForm,
      CommunityModerator,
      CommunityModeratorForm,
      CommunityPersonBan,
      CommunityPersonBanForm,
      CommunitySafe,
      CommunityUpdateForm,
    },
  },
  traits::{ApubActor, Bannable, Crud, DeleteableOrRemoveable, Followable, Joinable},
  utils::{functions::lower, get_conn, DbPool},
  SubscribedType,
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl, TextExpressionMethods};
use diesel_async::RunQueryDsl;

mod safe_type {
  use crate::{
    schema::community::{
      actor_id,
      banner,
      deleted,
      description,
      hidden,
      icon,
      id,
      instance_id,
      local,
      name,
      nsfw,
      posting_restricted_to_mods,
      published,
      removed,
      title,
      updated,
    },
    source::community::Community,
    traits::ToSafe,
  };

  type Columns = (
    id,
    name,
    title,
    description,
    removed,
    published,
    updated,
    deleted,
    nsfw,
    actor_id,
    local,
    icon,
    banner,
    hidden,
    posting_restricted_to_mods,
    instance_id,
  );

  impl ToSafe for Community {
    type SafeColumns = Columns;
    fn safe_columns_tuple() -> Self::SafeColumns {
      (
        id,
        name,
        title,
        description,
        removed,
        published,
        updated,
        deleted,
        nsfw,
        actor_id,
        local,
        icon,
        banner,
        hidden,
        posting_restricted_to_mods,
        instance_id,
      )
    }
  }
}

#[async_trait]
impl Crud for Community {
  type InsertForm = CommunityInsertForm;
  type UpdateForm = CommunityUpdateForm;
  type IdType = CommunityId;
  async fn read(pool: &DbPool, community_id: CommunityId) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    community.find(community_id).first::<Self>(conn).await
  }

  async fn delete(pool: &DbPool, community_id: CommunityId) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(community.find(community_id))
      .execute(conn)
      .await
  }

  async fn create(pool: &DbPool, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    let community_ = insert_into(community)
      .values(form)
      .on_conflict(actor_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await?;

    let site_languages = SiteLanguage::read_local(pool).await;
    if let Ok(langs) = site_languages {
      // if site exists, init user with site languages
      CommunityLanguage::update(pool, langs, community_.id).await?;
    } else {
      // otherwise, init with all languages (this only happens during tests)
      CommunityLanguage::update(pool, vec![], community_.id).await?;
    }

    Ok(community_)
  }

  async fn update(
    pool: &DbPool,
    community_id: CommunityId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(community.find(community_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

#[async_trait]
impl Joinable for CommunityModerator {
  type Form = CommunityModeratorForm;
  async fn join(
    pool: &DbPool,
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
    pool: &DbPool,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_moderator::dsl::{community_id, community_moderator, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      community_moderator
        .filter(community_id.eq(community_moderator_form.community_id))
        .filter(person_id.eq(community_moderator_form.person_id)),
    )
    .execute(conn)
    .await
  }
}

impl DeleteableOrRemoveable for CommunitySafe {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.title = String::new();
    self.description = None;
    self.icon = None;
    self.banner = None;
    self
  }
}

impl DeleteableOrRemoveable for Community {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.title = String::new();
    self.description = None;
    self.icon = None;
    self.banner = None;
    self
  }
}

pub enum CollectionType {
  Moderators,
  Featured,
}

impl Community {
  /// Get the community which has a given moderators or featured url, also return the collection type
  pub async fn get_by_collection_url(
    pool: &DbPool,
    url: &DbUrl,
  ) -> Result<(Community, CollectionType), Error> {
    use crate::schema::community::dsl::{featured_url, moderators_url};
    use CollectionType::*;
    let conn = &mut get_conn(pool).await?;
    let res = community
      .filter(moderators_url.eq(url))
      .first::<Self>(conn)
      .await;
    if let Ok(c) = res {
      return Ok((c, Moderators));
    }
    let res = community
      .filter(featured_url.eq(url))
      .first::<Self>(conn)
      .await;
    if let Ok(c) = res {
      return Ok((c, Featured));
    }
    Err(diesel::NotFound)
  }
}

impl CommunityModerator {
  pub async fn delete_for_community(
    pool: &DbPool,
    for_community_id: CommunityId,
  ) -> Result<usize, Error> {
    use crate::schema::community_moderator::dsl::{community_id, community_moderator};
    let conn = &mut get_conn(pool).await?;

    diesel::delete(community_moderator.filter(community_id.eq(for_community_id)))
      .execute(conn)
      .await
  }

  pub async fn get_person_moderated_communities(
    pool: &DbPool,
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
    pool: &DbPool,
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
    pool: &DbPool,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_person_ban::dsl::{community_id, community_person_ban, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      community_person_ban
        .filter(community_id.eq(community_person_ban_form.community_id))
        .filter(person_id.eq(community_person_ban_form.person_id)),
    )
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
}

#[async_trait]
impl Followable for CommunityFollower {
  type Form = CommunityFollowerForm;
  async fn follow(pool: &DbPool, form: &CommunityFollowerForm) -> Result<Self, Error> {
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
    pool: &DbPool,
    community_id_: CommunityId,
    person_id_: PersonId,
  ) -> Result<Self, Error> {
    use crate::schema::community_follower::dsl::{
      community_follower,
      community_id,
      pending,
      person_id,
    };
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      community_follower
        .filter(community_id.eq(community_id_))
        .filter(person_id.eq(person_id_)),
    )
    .set(pending.eq(false))
    .get_result::<Self>(conn)
    .await
  }
  async fn unfollow(pool: &DbPool, form: &CommunityFollowerForm) -> Result<usize, Error> {
    use crate::schema::community_follower::dsl::{community_follower, community_id, person_id};
    let conn = &mut get_conn(pool).await?;
    diesel::delete(
      community_follower
        .filter(community_id.eq(&form.community_id))
        .filter(person_id.eq(&form.person_id)),
    )
    .execute(conn)
    .await
  }
}

#[async_trait]
impl ApubActor for Community {
  async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      community
        .filter(actor_id.eq(object_id))
        .first::<Community>(conn)
        .await
        .ok()
        .map(Into::into),
    )
  }

  async fn read_from_name(
    pool: &DbPool,
    community_name: &str,
    include_deleted: bool,
  ) -> Result<Community, Error> {
    let conn = &mut get_conn(pool).await?;
    let mut q = community
      .into_boxed()
      .filter(local.eq(true))
      .filter(lower(name).eq(lower(community_name)));
    if !include_deleted {
      q = q.filter(deleted.eq(false)).filter(removed.eq(false));
    }
    q.first::<Self>(conn).await
  }

  async fn read_from_name_and_domain(
    pool: &DbPool,
    community_name: &str,
    protocol_domain: &str,
  ) -> Result<Community, Error> {
    let conn = &mut get_conn(pool).await?;
    community
      .filter(lower(name).eq(lower(community_name)))
      .filter(actor_id.like(format!("{protocol_domain}%")))
      .first::<Self>(conn)
      .await
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
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;

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
      id: inserted_community_follower.id,
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
      id: inserted_community_moderator.id,
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
      id: inserted_community_person_ban.id,
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      published: inserted_community_person_ban.published,
      expires: None,
    };

    let read_community = Community::read(pool, inserted_community.id).await.unwrap();

    let update_community_form = CommunityUpdateForm::builder()
      .title(Some("nada".to_owned()))
      .build();
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
