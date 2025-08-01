use crate::{
  diesel::OptionalExtension,
  newtypes::{ActivityId, DbUrl},
  source::activity::{ReceivedActivity, SentActivity, SentActivityForm},
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl SentActivity {
  pub async fn create(pool: &mut DbPool<'_>, form: SentActivityForm) -> LemmyResult<Self> {
    use lemmy_db_schema_file::schema::sent_activity::dsl::sent_activity;
    let conn = &mut get_conn(pool).await?;
    insert_into(sent_activity)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn read_from_apub_id(pool: &mut DbPool<'_>, object_id: &DbUrl) -> LemmyResult<Self> {
    use lemmy_db_schema_file::schema::sent_activity::dsl::{ap_id, sent_activity};
    let conn = &mut get_conn(pool).await?;
    sent_activity
      .filter(ap_id.eq(object_id))
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
  pub async fn read(pool: &mut DbPool<'_>, object_id: ActivityId) -> LemmyResult<Self> {
    use lemmy_db_schema_file::schema::sent_activity::dsl::sent_activity;
    let conn = &mut get_conn(pool).await?;
    sent_activity
      .find(object_id)
      .first(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl ReceivedActivity {
  pub async fn create(pool: &mut DbPool<'_>, ap_id_: &DbUrl) -> LemmyResult<()> {
    use lemmy_db_schema_file::schema::received_activity::dsl::{ap_id, received_activity};
    let conn = &mut get_conn(pool).await?;
    let rows_affected = insert_into(received_activity)
      .values(ap_id.eq(ap_id_))
      .on_conflict_do_nothing()
      .execute(conn)
      .await
      .optional()?;
    if rows_affected == Some(1) {
      // new activity inserted successfully
      Ok(())
    } else {
      Err(LemmyErrorType::CouldntCreate.into())
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::utils::build_db_pool_for_tests;
  use lemmy_db_schema_file::enums::ActorType;
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serde_json::json;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn receive_activity_duplicate() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let ap_id: DbUrl = Url::parse("http://example.com/activity/531")?.into();

    // inserting activity should only work once
    ReceivedActivity::create(pool, &ap_id).await?;
    let second = ReceivedActivity::create(pool, &ap_id).await;
    assert!(second.is_err());

    Ok(())
  }

  #[tokio::test]
  #[serial]
  async fn sent_activity_write_read() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();
    let ap_id: DbUrl = Url::parse("http://example.com/activity/412")?.into();
    let data = json!({
        "key1": "0xF9BA143B95FF6D82",
        "key2": "42",
    });
    let sensitive = false;

    let form = SentActivityForm {
      ap_id: ap_id.clone(),
      data: data.clone(),
      sensitive,
      actor_apub_id: Url::parse("http://example.com/u/exampleuser")?.into(),
      actor_type: ActorType::Person,
      send_all_instances: false,
      send_community_followers_of: None,
      send_inboxes: vec![],
    };

    SentActivity::create(pool, form).await?;

    let res = SentActivity::read_from_apub_id(pool, &ap_id).await?;
    assert_eq!(res.ap_id, ap_id);
    assert_eq!(res.data, data);
    assert_eq!(res.sensitive, sensitive);

    Ok(())
  }
}
