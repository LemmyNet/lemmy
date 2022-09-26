use crate::{newtypes::DbUrl, source::activity::*, traits::Crud};
use diesel::{
  dsl::*,
  result::{DatabaseErrorKind, Error},
  *,
};
use serde_json::Value;

impl Crud for Activity {
  type Form = ActivityForm;
  type IdType = i32;
  fn read(conn: &mut PgConnection, activity_id: i32) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    activity.find(activity_id).first::<Self>(conn)
  }

  fn create(conn: &mut PgConnection, new_activity: &ActivityForm) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &mut PgConnection,
    activity_id: i32,
    new_activity: &ActivityForm,
  ) -> Result<Self, Error> {
    use crate::schema::activity::dsl::*;
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
  }
  fn delete(conn: &mut PgConnection, activity_id: i32) -> Result<usize, Error> {
    use crate::schema::activity::dsl::*;
    diesel::delete(activity.find(activity_id)).execute(conn)
  }
}

impl Activity {
  /// Returns true if the insert was successful
  pub fn insert(
    conn: &mut PgConnection,
    ap_id: DbUrl,
    data: Value,
    local: bool,
    sensitive: bool,
  ) -> Result<bool, Error> {
    let activity_form = ActivityForm {
      ap_id,
      data,
      local: Some(local),
      sensitive,
      updated: None,
    };
    match Activity::create(conn, &activity_form) {
      Ok(_) => Ok(true),
      Err(e) => {
        if let Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
          return Ok(false);
        }
        Err(e)
      }
    }
  }

  pub fn read_from_apub_id(conn: &mut PgConnection, object_id: &DbUrl) -> Result<Activity, Error> {
    use crate::schema::activity::dsl::*;
    activity.filter(ap_id.eq(object_id)).first::<Self>(conn)
  }

  pub fn delete_olds(conn: &mut PgConnection) -> Result<usize, Error> {
    use crate::schema::activity::dsl::*;
    diesel::delete(activity.filter(published.lt(now - 6.months()))).execute(conn)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    newtypes::DbUrl,
    source::{
      activity::{Activity, ActivityForm},
      person::{Person, PersonForm},
    },
    utils::establish_unpooled_connection,
  };
  use serde_json::Value;
  use serial_test::serial;
  use url::Url;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let creator_form = PersonForm {
      name: "activity_creator_pm".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_creator = Person::create(conn, &creator_form).unwrap();

    let ap_id: DbUrl = Url::parse(
      "https://enterprise.lemmy.ml/activities/delete/f1b5d57c-80f8-4e03-a615-688d552e946c",
    )
    .unwrap()
    .into();
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
      ap_id: ap_id.clone(),
      data: test_json.to_owned(),
      local: Some(true),
      sensitive: false,
      updated: None,
    };

    let inserted_activity = Activity::create(conn, &activity_form).unwrap();

    let expected_activity = Activity {
      ap_id: ap_id.clone(),
      id: inserted_activity.id,
      data: test_json,
      local: true,
      sensitive: Some(false),
      published: inserted_activity.published,
      updated: None,
    };

    let read_activity = Activity::read(conn, inserted_activity.id).unwrap();
    let read_activity_by_apub_id = Activity::read_from_apub_id(conn, &ap_id).unwrap();
    Person::delete(conn, inserted_creator.id).unwrap();
    Activity::delete(conn, inserted_activity.id).unwrap();

    assert_eq!(expected_activity, read_activity);
    assert_eq!(expected_activity, read_activity_by_apub_id);
    assert_eq!(expected_activity, inserted_activity);
  }
}
