use crate::{schema::activity, Crud};
use diesel::{dsl::*, result::Error, *};
use log::debug;
use serde::Serialize;
use serde_json::Value;
use std::{
  fmt::Debug,
  io::{Error as IoError, ErrorKind},
};

#[derive(Queryable, Identifiable, PartialEq, Debug)]
#[table_name = "activity"]
pub struct Activity {
  pub id: i32,
  pub ap_id: String,
  pub data: Value,
  pub local: bool,
  pub sensitive: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable, AsChangeset)]
#[table_name = "activity"]
pub struct ActivityForm {
  pub ap_id: String,
  pub data: Value,
  pub local: bool,
  pub sensitive: bool,
  pub updated: Option<chrono::NaiveDateTime>,
}

impl Crud<ActivityForm> for Activity {
  fn read(conn: &PgConnection, activity_id: i32) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    activity.find(activity_id).first::<Self>(conn)
  }

  fn create(conn: &PgConnection, new_activity: &ActivityForm) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    activity_id: i32,
    new_activity: &ActivityForm,
  ) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
  }
}

impl Activity {
  pub fn insert<T>(
    conn: &PgConnection,
    ap_id: String,
    data: &T,
    local: bool,
    sensitive: bool,
  ) -> Result<Self, IoError>
  where
    T: Serialize + Debug,
  {
    debug!("{}", serde_json::to_string_pretty(&data)?);
    let activity_form = ActivityForm {
      ap_id,
      data: serde_json::to_value(&data)?,
      local,
      sensitive,
      updated: None,
    };
    let result = Activity::create(&conn, &activity_form);
    match result {
      Ok(s) => Ok(s),
      Err(e) => Err(IoError::new(
        ErrorKind::Other,
        format!("Failed to insert activity into database: {}", e),
      )),
    }
  }

  pub fn read_from_apub_id(conn: &PgConnection, object_id: &str) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    activity.filter(ap_id.eq(object_id)).first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    activity::{Activity, ActivityForm},
    tests::establish_unpooled_connection,
    user::{UserForm, User_},
    Crud,
    ListingType,
    SortType,
  };
  use serde_json::Value;

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
      banner: None,
      admin: false,
      banned: false,
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

    let inserted_creator = User_::create(&conn, &creator_form).unwrap();

    let ap_id =
      "https://enterprise.lemmy.ml/activities/delete/f1b5d57c-80f8-4e03-a615-688d552e946c";
    let test_json: Value = serde_json::from_str(
      r#"{
    "@context": "https://www.w3.org/ns/activitystreams",
    "id": "https://enterprise.lemmy.ml/activities/delete/f1b5d57c-80f8-4e03-a615-688d552e946c",
    "type": "Delete",
    "actor": "https://enterprise.lemmy.ml/u/riker",
    "to": "https://www.w3.org/ns/activitystreams#Public",
    "cc": [
        "https://enterprise.lemmy.ml/c/main/"
    ],
    "object": "https://enterprise.lemmy.ml/post/32"
    }"#,
    )
    .unwrap();
    let activity_form = ActivityForm {
      ap_id: ap_id.to_string(),
      data: test_json.to_owned(),
      local: true,
      sensitive: false,
      updated: None,
    };

    let inserted_activity = Activity::create(&conn, &activity_form).unwrap();

    let expected_activity = Activity {
      ap_id: ap_id.to_string(),
      id: inserted_activity.id,
      data: test_json,
      local: true,
      sensitive: false,
      published: inserted_activity.published,
      updated: None,
    };

    let read_activity = Activity::read(&conn, inserted_activity.id).unwrap();
    let read_activity_by_apub_id = Activity::read_from_apub_id(&conn, ap_id).unwrap();
    User_::delete(&conn, inserted_creator.id).unwrap();

    assert_eq!(expected_activity, read_activity);
    assert_eq!(expected_activity, read_activity_by_apub_id);
    assert_eq!(expected_activity, inserted_activity);
  }
}
