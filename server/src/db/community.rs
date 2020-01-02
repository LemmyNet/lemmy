use super::*;
use crate::schema::{community, community_follower, community_moderator, community_user_ban};

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
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
}

#[derive(Insertable, AsChangeset, Clone, Serialize, Deserialize)]
#[table_name = "community"]
pub struct CommunityForm {
  pub name: String,
  pub title: String,
  pub description: Option<String>,
  pub category_id: i32,
  pub creator_id: i32,
  pub removed: Option<bool>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub deleted: Option<bool>,
  pub nsfw: bool,
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
  pub fn read_from_name(conn: &PgConnection, community_name: String) -> Result<Self, Error> {
    use crate::schema::community::dsl::*;
    community
      .filter(name.eq(community_name))
      .first::<Self>(conn)
  }

  pub fn get_url(&self) -> String {
    format!("https://{}/c/{}", Settings::get().hostname, self.name)
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
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "community_follower"]
pub struct CommunityFollowerForm {
  pub community_id: i32,
  pub user_id: i32,
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
  fn ignore(
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
}

#[cfg(test)]
mod tests {
  use super::super::user::*;
  use super::*;
  #[test]
  fn test_crud() {
    let conn = establish_connection();

    let new_user = UserForm {
      name: "bobbee".into(),
      fedi_name: "rrf".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      avatar: None,
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
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
    };

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      user_id: inserted_user.id,
    };

    let inserted_community_follower =
      CommunityFollower::follow(&conn, &community_follower_form).unwrap();

    let expected_community_follower = CommunityFollower {
      id: inserted_community_follower.id,
      community_id: inserted_community.id,
      user_id: inserted_user.id,
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
    let ignored_community = CommunityFollower::ignore(&conn, &community_follower_form).unwrap();
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
