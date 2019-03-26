extern crate diesel;
use schema::{community, community_user, community_follower};
use diesel::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};
use {Crud, Followable, Joinable};

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name="community"]
pub struct Community {
  pub id: i32,
  pub name: String,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Insertable, AsChangeset, Clone, Serialize, Deserialize)]
#[table_name="community"]
pub struct CommunityForm {
  pub name: String,
  pub updated: Option<chrono::NaiveDateTime>
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_user"]
pub struct CommunityUser {
  pub id: i32,
  pub community_id: i32,
  pub fedi_user_id: String,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="community_user"]
pub struct CommunityUserForm {
  pub community_id: i32,
  pub fedi_user_id: String,
}

#[derive(Identifiable, Queryable, Associations, PartialEq, Debug)]
#[belongs_to(Community)]
#[table_name = "community_follower"]
pub struct CommunityFollower {
  pub id: i32,
  pub community_id: i32,
  pub fedi_user_id: String,
  pub published: chrono::NaiveDateTime,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name="community_follower"]
pub struct CommunityFollowerForm {
  pub community_id: i32,
  pub fedi_user_id: String,
}


impl Crud<CommunityForm> for Community {
  fn read(conn: &PgConnection, community_id: i32) -> Result<Self, Error> {
    use schema::community::dsl::*;
    community.find(community_id)
      .first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, community_id: i32) -> Result<usize, Error> {
    use schema::community::dsl::*;
    diesel::delete(community.find(community_id))
      .execute(conn)
  }

  fn create(conn: &PgConnection, new_community: &CommunityForm) -> Result<Self, Error> {
    use schema::community::dsl::*;
      insert_into(community)
        .values(new_community)
        .get_result::<Self>(conn)
  }

  fn update(conn: &PgConnection, community_id: i32, new_community: &CommunityForm) -> Result<Self, Error> {
    use schema::community::dsl::*;
    diesel::update(community.find(community_id))
      .set(new_community)
      .get_result::<Self>(conn)
  }
}

impl Followable<CommunityFollowerForm> for CommunityFollower {
  fn follow(conn: &PgConnection, community_follower_form: &CommunityFollowerForm) -> Result<Self, Error> {
    use schema::community_follower::dsl::*;
    insert_into(community_follower)
      .values(community_follower_form)
      .get_result::<Self>(conn)
  }
  fn ignore(conn: &PgConnection, community_follower_form: &CommunityFollowerForm) -> Result<usize, Error> {
    use schema::community_follower::dsl::*;
    diesel::delete(community_follower
      .filter(community_id.eq(&community_follower_form.community_id))
      .filter(fedi_user_id.eq(&community_follower_form.fedi_user_id)))
      .execute(conn)
  }
}

impl Joinable<CommunityUserForm> for CommunityUser {
  fn join(conn: &PgConnection, community_user_form: &CommunityUserForm) -> Result<Self, Error> {
    use schema::community_user::dsl::*;
    insert_into(community_user)
      .values(community_user_form)
      .get_result::<Self>(conn)
  }

  fn leave(conn: &PgConnection, community_user_form: &CommunityUserForm) -> Result<usize, Error> {
    use schema::community_user::dsl::*;
    diesel::delete(community_user
      .filter(community_id.eq(community_user_form.community_id))
      .filter(fedi_user_id.eq(&community_user_form.fedi_user_id)))
      .execute(conn)
  }
}

impl Community {
  pub fn list_all(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    use schema::community::dsl::*;
    community.load::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use establish_connection;
  use super::*;
  use actions::user::*;
  use Crud;
 #[test]
  fn test_crud() {
    let conn = establish_connection();
    
    let new_community = CommunityForm {
      name: "TIL".into(),
      updated: None
    };

    let inserted_community = Community::create(&conn, &new_community).unwrap();

    let expected_community = Community {
      id: inserted_community.id,
      name: "TIL".into(),
      published: inserted_community.published,
      updated: None
    };

    let new_user = UserForm {
      name: "terry".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      updated: None
    };

    let inserted_user = User_::create(&conn, &new_user).unwrap();

    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      fedi_user_id: "test".into()
    };

    let inserted_community_follower = CommunityFollower::follow(&conn, &community_follower_form).unwrap();

    let expected_community_follower = CommunityFollower {
      id: inserted_community_follower.id,
      community_id: inserted_community.id,
      fedi_user_id: "test".into(),
      published: inserted_community_follower.published
    };
    
    let community_user_form = CommunityUserForm {
      community_id: inserted_community.id,
      fedi_user_id: "test".into()
    };

    let inserted_community_user = CommunityUser::join(&conn, &community_user_form).unwrap();

    let expected_community_user = CommunityUser {
      id: inserted_community_user.id,
      community_id: inserted_community.id,
      fedi_user_id: "test".into(),
      published: inserted_community_user.published
    };

    let read_community = Community::read(&conn, inserted_community.id).unwrap();
    let updated_community = Community::update(&conn, inserted_community.id, &new_community).unwrap();
    let ignored_community = CommunityFollower::ignore(&conn, &community_follower_form).unwrap();
    let left_community = CommunityUser::leave(&conn, &community_user_form).unwrap();
    let loaded_count = Community::list_all(&conn).unwrap().len();
    let num_deleted = Community::delete(&conn, inserted_community.id).unwrap();
    User_::delete(&conn, inserted_user.id).unwrap();

    assert_eq!(expected_community, read_community);
    assert_eq!(expected_community, inserted_community);
    assert_eq!(expected_community, updated_community);
    assert_eq!(expected_community_follower, inserted_community_follower);
    assert_eq!(expected_community_user, inserted_community_user);
    assert_eq!(1, ignored_community);
    assert_eq!(1, left_community);
    assert_eq!(1, loaded_count);
    assert_eq!(1, num_deleted);

  }
}
