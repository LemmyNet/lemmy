use crate::Crud;
use diesel::{dsl::*, result::Error, sql_types::Text, *};
use lemmy_db_schema::{source::activity::*, DbUrl};
use log::debug;
use serde::Serialize;
use serde_json::Value;
use std::{
  fmt::Debug,
  io::{Error as IoError, ErrorKind},
};

impl Crud for Activity {
  type Form = ActivityForm;
  type IdType = i32;
  fn read(conn: &PgConnection, activity_id: i32) -> Result<Self, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    activity.find(activity_id).first::<Self>(conn)
  }

  fn create(conn: &PgConnection, new_activity: &ActivityForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    activity_id: i32,
    new_activity: &ActivityForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
  }
  fn delete(conn: &PgConnection, activity_id: i32) -> Result<usize, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    diesel::delete(activity.find(activity_id)).execute(conn)
  }
}

pub trait Activity_ {
  fn insert<T>(
    conn: &PgConnection,
    ap_id: DbUrl,
    data: &T,
    local: bool,
    sensitive: bool,
  ) -> Result<Activity, IoError>
  where
    T: Serialize + Debug;

  fn read_from_apub_id(conn: &PgConnection, object_id: &DbUrl) -> Result<Activity, Error>;
  fn delete_olds(conn: &PgConnection) -> Result<usize, Error>;

  /// Returns up to 20 activities of type `Announce/Create/Page` from the community
  fn read_community_outbox(
    conn: &PgConnection,
    community_actor_id: &DbUrl,
  ) -> Result<Vec<Value>, Error>;
}

impl Activity_ for Activity {
  fn insert<T>(
    conn: &PgConnection,
    ap_id: DbUrl,
    data: &T,
    local: bool,
    sensitive: bool,
  ) -> Result<Activity, IoError>
  where
    T: Serialize + Debug,
  {
    debug!("{}", serde_json::to_string_pretty(&data)?);
    let activity_form = ActivityForm {
      ap_id,
      data: serde_json::to_value(&data)?,
      local: Some(local),
      sensitive,
      updated: None,
    };
    let result = Activity::create(conn, &activity_form);
    match result {
      Ok(s) => Ok(s),
      Err(e) => Err(IoError::new(
        ErrorKind::Other,
        format!("Failed to insert activity into database: {}", e),
      )),
    }
  }

  fn read_from_apub_id(conn: &PgConnection, object_id: &DbUrl) -> Result<Activity, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    activity.filter(ap_id.eq(object_id)).first::<Self>(conn)
  }

  fn delete_olds(conn: &PgConnection) -> Result<usize, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    diesel::delete(activity.filter(published.lt(now - 6.months()))).execute(conn)
  }

  fn read_community_outbox(
    conn: &PgConnection,
    community_actor_id: &DbUrl,
  ) -> Result<Vec<Value>, Error> {
    use lemmy_db_schema::schema::activity::dsl::*;
    let res: Vec<Value> = activity
      .select(data)
      .filter(
        sql("activity.data ->> 'type' = 'Announce'")
          .sql(" AND activity.data -> 'object' ->> 'type' = 'Create'")
          .sql(" AND activity.data -> 'object' -> 'object' ->> 'type' = 'Page'")
          .sql(" AND activity.data ->> 'actor' = ")
          .bind::<Text, _>(community_actor_id)
          .sql(" ORDER BY activity.published DESC"),
      )
      .limit(20)
      .get_results(conn)?;
    Ok(res)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{establish_unpooled_connection, source::activity::Activity_};
  use lemmy_db_schema::source::{
    activity::{Activity, ActivityForm},
    person::{Person, PersonForm},
  };
  use serde_json::Value;
  use serial_test::serial;
  use url::Url;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let creator_form = PersonForm {
      name: "activity_creator_pm".into(),
      ..PersonForm::default()
    };

    let inserted_creator = Person::create(&conn, &creator_form).unwrap();

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

    let inserted_activity = Activity::create(&conn, &activity_form).unwrap();

    let expected_activity = Activity {
      ap_id: Some(ap_id.clone()),
      id: inserted_activity.id,
      data: test_json,
      local: true,
      sensitive: Some(false),
      published: inserted_activity.published,
      updated: None,
    };

    let read_activity = Activity::read(&conn, inserted_activity.id).unwrap();
    let read_activity_by_apub_id = Activity::read_from_apub_id(&conn, &ap_id).unwrap();
    Person::delete(&conn, inserted_creator.id).unwrap();
    Activity::delete(&conn, inserted_activity.id).unwrap();

    assert_eq!(expected_activity, read_activity);
    assert_eq!(expected_activity, read_activity_by_apub_id);
    assert_eq!(expected_activity, inserted_activity);
  }
}
