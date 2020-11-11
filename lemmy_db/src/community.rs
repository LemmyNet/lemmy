use crate::{
  naive_now,
  schema::{community, community_follower, community_moderator, community_user_ban},
  Bannable,
  Crud,
  Followable,
  Joinable,
};
use diesel::{dsl::*, result::Error, *};

#[derive(Clone, Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "community"]
pub struct Community {
  pub id: i32,
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub removed: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: bool,
  pub nsfw: bool,
  pub actor_id: String,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: chrono::NaiveDateTime,
  pub icon: Option<String>,
  pub banner: Option<String>,
}

#[derive(Insertable, AsChangeset, Debug)]
#[table_name = "community"]
pub struct CommunityForm {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub removed: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub nsfw: bool,
  pub actor_id: Option<String>,
  pub local: bool,
  pub private_key: Option<String>,
  pub public_key: Option<String>,
  pub last_refreshed_at: Option<chrono::NaiveDateTime>,
  pub icon: Option<Option<String>>,
  pub banner: Option<Option<String>>,
}

impl Crud<CommunityForm> for Community {
  fn read(conn: &PgConnection, community_id: i32) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    community.find(community_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, community_id: i32) -> Result<usize, Error> {
    use crate::schema::community::dsl::*;
    diesel::delete(community.find(community_id)).execute(conn)
  }

  fn create(conn: &PgConnection, new_community: &CommunityForm) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    insert_into(community)
      .values(new_community)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    community_id: i32,
    new_community: &CommunityForm,
  ) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set(new_community)
      .get_result::<Self>(conn)
  }
}

impl Community {
  pub fn read_from_name(conn: &PgConnection, community_name: &str) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    community
      .filter(local.eq(true))
      .filter(name.eq(community_name))
      .first::<Self>(conn)
  }

  pub fn read_from_actor_id(conn: &PgConnection, for_actor_id: &str) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    community
      .filter(actor_id.eq(for_actor_id))
      .first::<Self>(conn)
  }

  pub fn update_deleted(
    conn: &PgConnection,
    community_id: i32,
    new_deleted: bool,
  ) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set((deleted.eq(new_deleted), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed(
    conn: &PgConnection,
    community_id: i32,
    new_removed: bool,
  ) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_removed_for_creator(
    conn: &PgConnection,
    for_creator_id: i32,
    new_removed: bool,
  ) -> Result<Vec<Self>, Error> {
    use crate::schema::community::dsl::*;
    diesel::update(community.filter(creator_id.eq(for_creator_id)))
      .set((removed.eq(new_removed), updated.eq(naive_now())))
      .get_results::<Self>(conn)
  }

  pub fn update_creator(
    conn: &PgConnection,
    community_id: i32,
    new_creator_id: i32,
  ) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set((creator_id.eq(new_creator_id), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  fn community_mods_and_admins(conn: &PgConnection, community_id: i32) -> Result<Vec<i32>, Error> {
    use crate::{community_view::CommunityModeratorView, user_view::UserView};
    let mut mods_and_admins: Vec<i32> = Vec::new();
    mods_and_admins.append(
      &mut CommunityModeratorView::for_community(conn, community_id)
        .map(|v| v.into_iter().map(|m| m.user_id).collect())?,
    );
    mods_and_admins
      .append(&mut UserView::admins(conn).map(|v| v.into_iter().map(|a| a.id).collect())?);
    Ok(mods_and_admins)
  }

  pub fn distinct_federated_communities(conn: &PgConnection) -> Result<Vec<String>, Error> {
    use crate::schema::community::dsl::*;
    community.select(actor_id).distinct().load::<String>(conn)
  }

  pub fn is_mod_or_admin(conn: &PgConnection, user_id: i32, community_id: i32) -> bool {
    Self::community_mods_and_admins(conn, community_id)
      .unwrap_or_default()
      .contains(&user_id)
  }

  pub fn upsert(conn: &PgConnection, community_form: &CommunityForm) -> Result<Community, Error> {
    use crate::schema::community::dsl::*;
    insert_into(community)
      .values(community_form)
      .on_conflict(actor_id)
      .do_update()
      .set(community_form)
      .get_result::<Self>(conn)
  }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_moderator"]
pub struct CommunityModerator {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_moderator"]
pub struct CommunityModeratorForm {
  pub community_id: i32,
  pub user_id: i32,
}

impl Joinable<CommunityModeratorForm> for CommunityModerator {
  fn join(
    conn: &PgConnection,
    community_user_form: &CommunityModeratorForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_moderator::dsl::*;
    insert_into(community_moderator)
      .values(community_user_form)
      .get_result::<Self>(conn)
  }

  fn leave(
    conn: &PgConnection,
    community_user_form: &CommunityModeratorForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_moderator::dsl::*;
    diesel::delete(
      community_moderator
        .filter(community_id.eq(community_user_form.community_id))
        .filter(user_id.eq(community_user_form.user_id)),
    )
    .execute(conn)
  }
}

impl CommunityModerator {
  pub fn delete_for_community(conn: &PgConnection, for_community_id: i32) -> Result<usize, Error> {
    use crate::schema::community_moderator::dsl::*;
    diesel::delete(community_moderator.filter(community_id.eq(for_community_id))).execute(conn)
  }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_user_ban"]
pub struct CommunityUserBan {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_user_ban"]
pub struct CommunityUserBanForm {
  pub community_id: i32,
  pub user_id: i32,
}

impl Bannable<CommunityUserBanForm> for CommunityUserBan {
  fn ban(
    conn: &PgConnection,
    community_user_ban_form: &CommunityUserBanForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_user_ban::dsl::*;
    insert_into(community_user_ban)
      .values(community_user_ban_form)
      .get_result::<Self>(conn)
  }

  fn unban(
    conn: &PgConnection,
    community_user_ban_form: &CommunityUserBanForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_user_ban::dsl::*;
    diesel::delete(
      community_user_ban
        .filter(community_id.eq(community_user_ban_form.community_id))
        .filter(user_id.eq(community_user_ban_form.user_id)),
    )
    .execute(conn)
  }
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_follower"]
pub struct CommunityFollower {
  pub id: i32,
  pub community_id: i32,
  pub user_id: i32,
  pub published: chrono::NaiveDateTime,
  pub pending: Option<bool>,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_follower"]
pub struct CommunityFollowerForm {
  pub community_id: i32,
  pub user_id: i32,
  pub pending: bool,
}

impl Followable<CommunityFollowerForm> for CommunityFollower {
  fn follow(
    conn: &PgConnection,
    community_follower_form: &CommunityFollowerForm,
  ) -> Result<Self, Error> {
    use crate::schema::community_follower::dsl::*;
    insert_into(community_follower)
      .values(community_follower_form)
      .get_result::<Self>(conn)
  }
  fn follow_accepted(conn: &PgConnection, community_id_: i32, user_id_: i32) -> Result<Self, Error>
  where
    Self: Sized,
  {
    use crate::schema::community_follower::dsl::*;
    diesel::update(
      community_follower
        .filter(community_id.eq(community_id_))
        .filter(user_id.eq(user_id_)),
    )
    .set(pending.eq(true))
    .get_result::<Self>(conn)
  }
  fn unfollow(
    conn: &PgConnection,
    community_follower_form: &CommunityFollowerForm,
  ) -> Result<usize, Error> {
    use crate::schema::community_follower::dsl::*;
    diesel::delete(
      community_follower
        .filter(community_id.eq(&community_follower_form.community_id))
        .filter(user_id.eq(&community_follower_form.user_id)),
    )
    .execute(conn)
  }
  // TODO: this function name only makes sense if you call it with a remote community. for a local
  //       community, it will also return true if only remote followers exist
  fn has_local_followers(conn: &PgConnection, community_id_: i32) -> Result<bool, Error> {
    use crate::schema::community_follower::dsl::*;
    diesel::select(exists(
      community_follower.filter(community_id.eq(community_id_)),
    ))
    .get_result(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{community::*, tests::establish_unpooled_connection, user::*, ListingType, SortType};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let new_user = UserForm {
      name: "bobbee".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
      published: None,
      updated: None,
      show_nsfw: false,
      theme: "browser".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: None,
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let new_community = CommunityForm {
      name: "TIL".into(),
      creator_id: inserted_user.id,
      title: "nada".to_owned(),
      description: None,
      category_id: 1,
      nsfw: false,
      removed: None,
      deleted: None,
      updated: None,
      actor_id: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
      published: None,
      icon: None,
      banner: None,
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let expected_community = Community {
      id: inserted_community.id,
      creator_id: inserted_user.id,
      name: "TIL".into(),
      title: "nada".to_owned(),
      description: None,
      category_id: 1,
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
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: inserted_user.id,
      pending: false,
    };

    let inserted_community_follower =
      CommunityFollower::follow(&conn, &community_follower_form).unwrap();

    let expected_community_follower = CommunityFollower {
      id: inserted_community_follower.id,
      community_id: inserted_community.id,
      user_id: inserted_user.id,
      pending: Some(false),
      published: inserted_community_follower.published,
    };

    let community_user_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      user_id: inserted_user.id,
    };

    let inserted_community_user = CommunityModerator::join(&conn, &community_user_form).unwrap();

    let expected_community_user = CommunityModerator {
      id: inserted_community_user.id,
      community_id: inserted_community.id,
      user_id: inserted_user.id,
      published: inserted_community_user.published,
    };

    let community_user_ban_form = CommunityUserBanForm {
      community_id: inserted_community.id,
      user_id: inserted_user.id,
    };

    let inserted_community_user_ban =
      CommunityUserBan::ban(&conn, &community_user_ban_form).unwrap();

    let expected_community_user_ban = CommunityUserBan {
      id: inserted_community_user_ban.id,
      community_id: inserted_community.id,
      user_id: inserted_user.id,
      published: inserted_community_user_ban.published,
    };

    let read_community = Community::read(&conn, inserted_community.id).unwrap();
    let updated_community =
      Community::update(&conn, inserted_community.id, &new_community).unwrap();
    let ignored_community = CommunityFollower::unfollow(&conn, &community_follower_form).unwrap();
    let left_community = CommunityModerator::leave(&conn, &community_user_form).unwrap();
    let unban = CommunityUserBan::unban(&conn, &community_user_ban_form).unwrap();
    let num_deleted = Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_community, read_community);
    assert_eq!(expected_community, inserted_community);
    assert_eq!(expected_community, updated_community);
    assert_eq!(expected_community_follower, inserted_community_follower);
    assert_eq!(expected_community_user, inserted_community_user);
    assert_eq!(expected_community_user_ban, inserted_community_user_ban);
    assert_eq!(1, ignored_community);
    assert_eq!(1, left_community);
    assert_eq!(1, unban);
    // assert_eq!(2, loaded_count);
    assert_eq!(1, num_deleted);
  }
}
