use crate::{
  newtypes::DbUrl,
  schema::activity::dsl::{activity, ap_id},
  source::activity::{Activity, ActivityInsertForm, ActivityUpdateForm},
  traits::Crud,
  utils::DbConn,
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for Activity {
  type InsertForm = ActivityInsertForm;
  type UpdateForm = ActivityUpdateForm;
  type IdType = i32;
  async fn read(mut conn: impl DbConn, activity_id: i32) -> Result<Self, Error> {
    activity.find(activity_id).first::<Self>(conn).await
  }

  async fn create(mut conn: impl DbConn, new_activity: &Self::InsertForm) -> Result<Self, Error> {
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    mut conn: impl DbConn,
    activity_id: i32,
    new_activity: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
      .await
  }
  async fn delete(mut conn: impl DbConn, activity_id: i32) -> Result<usize, Error> {
    diesel::delete(activity.find(activity_id))
      .execute(conn)
      .await
  }
}

impl Activity {
  pub async fn read_from_apub_id(
    mut conn: impl DbConn,
    object_id: &DbUrl,
  ) -> Result<Activity, Error> {
    activity
      .filter(ap_id.eq(object_id))
      .first::<Self>(conn)
      .await
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    newtypes::DbUrl,
    source::{
      activity::{Activity, ActivityInsertForm},
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    utils::build_db_conn_for_tests,
  };
  use serde_json::Value;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let mut conn = build_db_conn_for_tests().await;

    let inserted_instance = Instance::read_or_create(&mut *conn, "my_domain.tld".to_string())
      .await
      .unwrap();

    let creator_form = PersonInsertForm::builder()
      .name("activity_creator_ pm".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_creator = Person::create(&mut *conn, &creator_form).await.unwrap();

    let ap_id_: DbUrl = Url::parse(
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
    let activity_form = ActivityInsertForm {
      ap_id: ap_id_.clone(),
      data: test_json.clone(),
      local: Some(true),
      sensitive: Some(false),
      updated: None,
    };

    let inserted_activity = Activity::create(&mut *conn, &activity_form).await.unwrap();

    let expected_activity = Activity {
      ap_id: ap_id_.clone(),
      id: inserted_activity.id,
      data: test_json,
      local: true,
      sensitive: false,
      published: inserted_activity.published,
      updated: None,
    };

    let read_activity = Activity::read(&mut *conn, inserted_activity.id)
      .await
      .unwrap();
    let read_activity_by_apub_id = Activity::read_from_apub_id(&mut *conn, &ap_id_)
      .await
      .unwrap();
    Person::delete(&mut *conn, inserted_creator.id)
      .await
      .unwrap();
    Activity::delete(&mut *conn, inserted_activity.id)
      .await
      .unwrap();

    assert_eq!(expected_activity, read_activity);
    assert_eq!(expected_activity, read_activity_by_apub_id);
    assert_eq!(expected_activity, inserted_activity);
  }
}
