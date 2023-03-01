use crate::{
  newtypes::DbUrl,
  schema::activity::dsl::{activity, ap_id},
  source::activity::{Activity, ActivityInsertForm, ActivityUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{
  dsl::insert_into,
  result::{DatabaseErrorKind, Error},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use serde_json::Value;

#[async_trait]
impl Crud for Activity {
  type InsertForm = ActivityInsertForm;
  type UpdateForm = ActivityUpdateForm;
  type IdType = i32;
  async fn read(pool: &DbPool, activity_id: i32) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    activity.find(activity_id).first::<Self>(conn).await
  }

  async fn create(pool: &DbPool, new_activity: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &DbPool,
    activity_id: i32,
    new_activity: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
      .await
  }
  async fn delete(pool: &DbPool, activity_id: i32) -> Result<usize, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::delete(activity.find(activity_id))
      .execute(conn)
      .await
  }
}

impl Activity {
  /// Returns true if the insert was successful
  // TODO this should probably just be changed to an upsert on_conflict, rather than an error
  pub async fn insert(
    pool: &DbPool,
    ap_id_: DbUrl,
    data_: Value,
    local_: bool,
    sensitive_: Option<bool>,
  ) -> Result<bool, Error> {
    let activity_form = ActivityInsertForm {
      ap_id: ap_id_,
      data: data_,
      local: Some(local_),
      sensitive: sensitive_,
      updated: None,
    };
    match Activity::create(pool, &activity_form).await {
      Ok(_) => Ok(true),
      Err(e) => {
        if let Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) = e {
          return Ok(false);
        }
        Err(e)
      }
    }
  }

  pub async fn read_from_apub_id(pool: &DbPool, object_id: &DbUrl) -> Result<Activity, Error> {
    let conn = &mut get_conn(pool).await?;
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
    utils::build_db_pool_for_tests,
  };
  use serde_json::Value;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let pool = &build_db_pool_for_tests().await;

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let creator_form = PersonInsertForm::builder()
      .name("activity_creator_ pm".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_creator = Person::create(pool, &creator_form).await.unwrap();

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

    let inserted_activity = Activity::create(pool, &activity_form).await.unwrap();

    let expected_activity = Activity {
      ap_id: ap_id_.clone(),
      id: inserted_activity.id,
      data: test_json,
      local: true,
      sensitive: Some(false),
      published: inserted_activity.published,
      updated: None,
    };

    let read_activity = Activity::read(pool, inserted_activity.id).await.unwrap();
    let read_activity_by_apub_id = Activity::read_from_apub_id(pool, &ap_id_).await.unwrap();
    Person::delete(pool, inserted_creator.id).await.unwrap();
    Activity::delete(pool, inserted_activity.id).await.unwrap();

    assert_eq!(expected_activity, read_activity);
    assert_eq!(expected_activity, read_activity_by_apub_id);
    assert_eq!(expected_activity, inserted_activity);
  }
}
