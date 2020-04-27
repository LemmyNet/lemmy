use super::*;
use crate::schema::activity;
use crate::schema::activity::dsl::*;
use serde_json::Value;

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "activity"]
pub struct Activity {
  pub id: i32,
  pub user_id: i32,
  pub data: Value,
  pub local: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset, Clone, Serialize, Deserialize)]
#[table_name = "activity"]
pub struct ActivityForm {
  pub user_id: i32,
  pub data: Value,
  pub local: bool,
  pub updated: Option<chrono::NaiveDateTime>,
}

impl Crud<ActivityForm> for Activity {
  fn read(conn: &PgConnection, activity_id: i32) -> Result<Self, Error> {
    activity.find(activity_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, activity_id: i32) -> Result<usize, Error> {
    diesel::delete(activity.find(activity_id)).execute(conn)
  }

  fn create(conn: &PgConnection, new_activity: &ActivityForm) -> Result<Self, Error> {
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    activity_id: i32,
    new_activity: &ActivityForm,
  ) -> Result<Self, Error> {
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use super::super::user::*;
  use super::*;

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let creator_form = UserForm {
      name: "activity_creator_pm".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
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
      actor_id: "changeme".into(),
      bio: None,
      local: true,
      private_key: None,
      public_key: None,
      last_refreshed_at: None,
    };

    let inserted_creator = User_::create(&conn, &creator_form).unwrap();

    let test_json: Value = serde_json::from_str(
      r#"{
    "street": "Article Circle Expressway 1",
    "city": "North Pole",
    "postcode": "99705",
    "state": "Alaska"
}"#,
    )
    .unwrap();
    let activity_form = ActivityForm {
      user_id: inserted_creator.id,
      data: test_json.to_owned(),
      local: true,
      updated: None,
    };

    let inserted_activity = Activity::create(&conn, &activity_form).unwrap();

    let expected_activity = Activity {
      id: inserted_activity.id,
      user_id: inserted_creator.id,
      data: test_json,
      local: true,
      published: inserted_activity.published,
      updated: None,
    };

    let read_activity = Activity::read(&conn, inserted_activity.id).unwrap();
    let num_deleted = Activity::delete(&conn, inserted_activity.id).unwrap();
    User_::delete(&conn, inserted_creator.id).unwrap();

    assert_eq!(expected_activity, read_activity);
    assert_eq!(expected_activity, inserted_activity);
    assert_eq!(1, num_deleted);
  }
}
