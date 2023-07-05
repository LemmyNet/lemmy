use crate::{
  newtypes::DbUrl,
  schema::activity::dsl::{activity, ap_id},
  source::activity::{Activity, ActivityInsertForm, ActivityUpdateForm},
  traits::Crud,
  utils::{DbPool, DbPoolRef, RunQueryDsl},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};

#[async_trait]
impl Crud for Activity {
  type InsertForm = ActivityInsertForm;
  type UpdateForm = ActivityUpdateForm;
  type IdType = i32;
  async fn read(pool: DbPoolRef<'_>, activity_id: i32) -> Result<Self, Error> {
    let conn = pool;
    activity.find(activity_id).first::<Self>(conn).await
  }

  async fn create(pool: DbPoolRef<'_>, new_activity: &Self::InsertForm) -> Result<Self, Error> {
    let conn = pool;
    insert_into(activity)
      .values(new_activity)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: DbPoolRef<'_>,
    activity_id: i32,
    new_activity: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = pool;
    diesel::update(activity.find(activity_id))
      .set(new_activity)
      .get_result::<Self>(conn)
      .await
  }
  async fn delete(pool: DbPoolRef<'_>, activity_id: i32) -> Result<usize, Error> {
    let conn = pool;
    diesel::delete(activity.find(activity_id))
      .execute(conn)
      .await
  }
}

impl Activity {
  pub async fn read_from_apub_id(
    pool: DbPoolRef<'_>,
    object_id: &DbUrl,
  ) -> Result<Activity, Error> {
    let conn = pool;
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
    let mut pool = &mut crate::utils::DbPool::Pool(&build_db_pool_for_tests().await);

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
      sensitive: false,
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
