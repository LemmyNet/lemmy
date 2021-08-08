use crate::{ApubObject, Bannable, Crud, DeleteableOrRemoveable, Followable, Joinable};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{
  naive_now,
  source::community::{
    Community,
    CommunityFollower,
    CommunityFollowerForm,
    CommunityForm,
    CommunityModerator,
    CommunityModeratorForm,
    CommunityPersonBan,
    CommunityPersonBanForm,
    CommunitySafe,
  },
  CommunityId,
  DbUrl,
  PersonId,
};

mod safe_type {
  use crate::{source::community::Community, ToSafe};
  use lemmy_db_schema::schema::community::*;

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
      )
    }
  }
}

impl Crud for Community {
  type Form = CommunityForm;
  type IdType = CommunityId;
  fn read(conn: &PgConnection, community_id: CommunityId) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    community.find(community_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, community_id: CommunityId) -> Result<usize, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    diesel::delete(community.find(community_id)).execute(conn)
  }

  fn create(conn: &PgConnection, new_community: &CommunityForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    insert_into(community)
      .values(new_community)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    community_id: CommunityId,
    new_community: &CommunityForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set(new_community)
      .get_result::<Self>(conn)
  }
}

impl ApubObject for Community {
  type Form = CommunityForm;
  fn read_from_apub_id(conn: &PgConnection, for_actor_id: &DbUrl) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    community
      .filter(actor_id.eq(for_actor_id))
      .first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, community_form: &CommunityForm) -> Result<Community, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    insert_into(community)
      .values(community_form)
      .on_conflict(actor_id)
      .do_update()
      .set(community_form)
      .get_result::<Self>(conn)
  }
}

pub trait Community_ {
  fn read_from_name(conn: &PgConnection, community_name: &str) -> Result<Community, Error>;
  fn update_deleted(
    conn: &PgConnection,
    community_id: CommunityId,
    new_deleted: bool,
  ) -> Result<Community, Error>;
  fn update_removed(
    conn: &PgConnection,
    community_id: CommunityId,
    new_removed: bool,
  ) -> Result<Community, Error>;
  fn distinct_federated_communities(conn: &PgConnection) -> Result<Vec<String>, Error>;
  fn read_from_followers_url(
    conn: &PgConnection,
    followers_url: &DbUrl,
  ) -> Result<Community, Error>;
}

impl Community_ for Community {
  fn read_from_name(conn: &PgConnection, community_name: &str) -> Result<Community, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    community
      .filter(local.eq(true))
      .filter(name.eq(community_name))
      .first::<Self>(conn)
  }

  fn update_deleted(
    conn: &PgConnection,
    community_id: CommunityId,
    new_deleted: bool,
  ) -> Result<Community, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  fn update_removed(
    conn: &PgConnection,
    community_id: CommunityId,
    new_removed: bool,
  ) -> Result<Community, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  fn distinct_federated_communities(conn: &PgConnection) -> Result<Vec<String>, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    community.select(actor_id).distinct().load::<String>(conn)
  }

  fn read_from_followers_url(
    conn: &PgConnection,
    followers_url_: &DbUrl,
  ) -> Result<Community, Error> {
    use lemmy_db_schema::schema::community::dsl::*;
    community
      .filter(followers_url.eq(followers_url_))
      .first::<Self>(conn)
  }
}

impl Joinable for CommunityModerator {
  type Form = CommunityModeratorForm;
  fn join(
    conn: &PgConnection,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community_moderator::dsl::*;
    insert_into(community_moderator)
      .values(community_moderator_form)
      .get_result::<Self>(conn)
  }

  fn leave(
    conn: &PgConnection,
    community_moderator_form: &CommunityModeratorForm,
  ) -> Result<usize, Error> {
    use lemmy_db_schema::schema::community_moderator::dsl::*;
    diesel::delete(
      community_moderator
        .filter(community_id.eq(community_moderator_form.community_id))
        .filter(person_id.eq(community_moderator_form.person_id)),
    )
    .execute(conn)
  }
}

impl DeleteableOrRemoveable for CommunitySafe {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.title = "".into();
    self.description = None;
    self.icon = None;
    self.banner = None;
    self
  }
}

impl DeleteableOrRemoveable for Community {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.title = "".into();
    self.description = None;
    self.icon = None;
    self.banner = None;
    self
  }
}

pub trait CommunityModerator_ {
  fn delete_for_community(
    conn: &PgConnection,
    for_community_id: CommunityId,
  ) -> Result<usize, Error>;
  fn get_person_moderated_communities(
    conn: &PgConnection,
    for_person_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error>;
}

impl CommunityModerator_ for CommunityModerator {
  fn delete_for_community(
    conn: &PgConnection,
    for_community_id: CommunityId,
  ) -> Result<usize, Error> {
    use lemmy_db_schema::schema::community_moderator::dsl::*;
    diesel::delete(community_moderator.filter(community_id.eq(for_community_id))).execute(conn)
  }

  fn get_person_moderated_communities(
    conn: &PgConnection,
    for_person_id: PersonId,
  ) -> Result<Vec<CommunityId>, Error> {
    use lemmy_db_schema::schema::community_moderator::dsl::*;
    community_moderator
      .filter(person_id.eq(for_person_id))
      .select(community_id)
      .load::<CommunityId>(conn)
  }
}

impl Bannable for CommunityPersonBan {
  type Form = CommunityPersonBanForm;
  fn ban(
    conn: &PgConnection,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community_person_ban::dsl::*;
    insert_into(community_person_ban)
      .values(community_person_ban_form)
      .get_result::<Self>(conn)
  }

  fn unban(
    conn: &PgConnection,
    community_person_ban_form: &CommunityPersonBanForm,
  ) -> Result<usize, Error> {
    use lemmy_db_schema::schema::community_person_ban::dsl::*;
    diesel::delete(
      community_person_ban
        .filter(community_id.eq(community_person_ban_form.community_id))
        .filter(person_id.eq(community_person_ban_form.person_id)),
    )
    .execute(conn)
  }
}

impl Followable for CommunityFollower {
  type Form = CommunityFollowerForm;
  fn follow(
    conn: &PgConnection,
    community_follower_form: &CommunityFollowerForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::community_follower::dsl::*;
    insert_into(community_follower)
      .values(community_follower_form)
      .on_conflict((community_id, person_id))
      .do_update()
      .set(community_follower_form)
      .get_result::<Self>(conn)
  }
  fn follow_accepted(
    conn: &PgConnection,
    community_id_: CommunityId,
    person_id_: PersonId,
  ) -> Result<Self, Error>
  where
    Self: Sized,
  {
    use lemmy_db_schema::schema::community_follower::dsl::*;
    diesel::update(
      community_follower
        .filter(community_id.eq(community_id_))
        .filter(person_id.eq(person_id_)),
    )
    .set(pending.eq(true))
    .get_result::<Self>(conn)
  }
  fn unfollow(
    conn: &PgConnection,
    community_follower_form: &CommunityFollowerForm,
  ) -> Result<usize, Error> {
    use lemmy_db_schema::schema::community_follower::dsl::*;
    diesel::delete(
      community_follower
        .filter(community_id.eq(&community_follower_form.community_id))
        .filter(person_id.eq(&community_follower_form.person_id)),
    )
    .execute(conn)
  }
  // TODO: this function name only makes sense if you call it with a remote community. for a local
  //       community, it will also return true if only remote followers exist
  fn has_local_followers(conn: &PgConnection, community_id_: CommunityId) -> Result<bool, Error> {
    use lemmy_db_schema::schema::community_follower::dsl::*;
    diesel::select(exists(
      community_follower.filter(community_id.eq(community_id_)),
    ))
    .get_result(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{establish_unpooled_connection, Bannable, Crud, Followable, Joinable};
  use lemmy_db_schema::source::{community::*, person::*};
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_person = PersonForm {
      name: "bobbee".into(),
      ..PersonForm::default()
    };

    let inserted_person = Person::create(&conn, &new_person).unwrap();

    let new_community = CommunityForm {
      name: "TIL".into(),
      title: "nada".to_owned(),
      ..CommunityForm::default()
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

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
      actor_id: inserted_community.actor_id.to_owned(),
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: inserted_community.published,
      icon: None,
      banner: None,
      followers_url: inserted_community.followers_url.to_owned(),
      inbox_url: inserted_community.inbox_url.to_owned(),
      shared_inbox_url: None,
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      pending: false,
    };

    let inserted_community_follower =
      CommunityFollower::follow(&conn, &community_follower_form).unwrap();

    let expected_community_follower = CommunityFollower {
      id: inserted_community_follower.id,
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      pending: Some(false),
      published: inserted_community_follower.published,
    };

    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
    };

    let inserted_community_moderator =
      CommunityModerator::join(&conn, &community_moderator_form).unwrap();

    let expected_community_moderator = CommunityModerator {
      id: inserted_community_moderator.id,
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      published: inserted_community_moderator.published,
    };

    let community_person_ban_form = CommunityPersonBanForm {
      community_id: inserted_community.id,
      person_id: inserted_person.id,
    };

    let inserted_community_person_ban =
      CommunityPersonBan::ban(&conn, &community_person_ban_form).unwrap();

    let expected_community_person_ban = CommunityPersonBan {
      id: inserted_community_person_ban.id,
      community_id: inserted_community.id,
      person_id: inserted_person.id,
      published: inserted_community_person_ban.published,
    };

    let read_community = Community::read(&conn, inserted_community.id).unwrap();
    let updated_community =
      Community::update(&conn, inserted_community.id, &new_community).unwrap();
    let ignored_community = CommunityFollower::unfollow(&conn, &community_follower_form).unwrap();
    let left_community = CommunityModerator::leave(&conn, &community_moderator_form).unwrap();
    let unban = CommunityPersonBan::unban(&conn, &community_person_ban_form).unwrap();
    let num_deleted = Community::delete(&conn, inserted_community.id).unwrap();
    Person::delete(&conn, inserted_person.id).unwrap();

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
