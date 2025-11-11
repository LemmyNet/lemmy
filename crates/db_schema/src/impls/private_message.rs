use crate::{
  diesel::{DecoratableTarget, OptionalExtension},
  newtypes::PrivateMessageId,
  source::private_message::{PrivateMessage, PrivateMessageInsertForm, PrivateMessageUpdateForm},
};
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, dsl::insert_into};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::{PersonId, schema::private_message};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  dburl::DbUrl,
  traits::Crud,
  utils::functions::coalesce,
};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
  settings::structs::Settings,
};
use url::Url;

impl Crud for PrivateMessage {
  type InsertForm = PrivateMessageInsertForm;
  type UpdateForm = PrivateMessageUpdateForm;
  type IdType = PrivateMessageId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(private_message::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    private_message_id: PrivateMessageId,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(private_message::table.find(private_message_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl PrivateMessage {
  pub async fn insert_apub(
    pool: &mut DbPool<'_>,
    timestamp: DateTime<Utc>,
    form: &PrivateMessageInsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(private_message::table)
      .values(form)
      .on_conflict(private_message::ap_id)
      .filter_target(
        coalesce(private_message::updated_at, private_message::published_at).lt(timestamp),
      )
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn read_from_apub_id(
    pool: &mut DbPool<'_>,
    object_id: DbUrl,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    private_message::table
      .filter(private_message::ap_id.eq(object_id))
      .first(conn)
      .await
      .optional()
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
  pub fn local_url(&self, settings: &Settings) -> LemmyResult<DbUrl> {
    let domain = settings.get_protocol_and_hostname();
    Ok(Url::parse(&format!("{domain}/private_message/{}", self.id))?.into())
  }

  pub async fn update_removed_for_creator(
    pool: &mut DbPool<'_>,
    for_creator_id: PersonId,
    removed: bool,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(private_message::table.filter(private_message::creator_id.eq(for_creator_id)))
      .set((
        private_message::removed.eq(removed),
        private_message::updated_at.eq(Utc::now()),
      ))
      .get_results::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

#[cfg(test)]
mod tests {

  use crate::source::{
    instance::Instance,
    person::{Person, PersonInsertForm},
    private_message::{PrivateMessage, PrivateMessageInsertForm, PrivateMessageUpdateForm},
  };
  use lemmy_diesel_utils::{connection::build_db_pool_for_tests, traits::Crud};
  use lemmy_utils::error::LemmyResult;
  use pretty_assertions::assert_eq;
  use serial_test::serial;
  use url::Url;

  #[tokio::test]
  #[serial]
  async fn test_crud() -> LemmyResult<()> {
    let pool = &build_db_pool_for_tests();
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld").await?;

    let creator_form = PersonInsertForm::test_form(inserted_instance.id, "creator_pm");

    let inserted_creator = Person::create(pool, &creator_form).await?;

    let recipient_form = PersonInsertForm::test_form(inserted_instance.id, "recipient_pm");

    let inserted_recipient = Person::create(pool, &recipient_form).await?;

    let private_message_form = PrivateMessageInsertForm::new(
      inserted_creator.id,
      inserted_recipient.id,
      "A test private message".into(),
    );

    let inserted_private_message = PrivateMessage::create(pool, &private_message_form).await?;

    let expected_private_message = PrivateMessage {
      id: inserted_private_message.id,
      content: "A test private message".into(),
      creator_id: inserted_creator.id,
      recipient_id: inserted_recipient.id,
      deleted: false,
      updated_at: None,
      published_at: inserted_private_message.published_at,
      ap_id: Url::parse(&format!(
        "https://lemmy-alpha/private_message/{}",
        inserted_private_message.id
      ))?
      .into(),
      local: true,
      removed: false,
    };

    let read_private_message = PrivateMessage::read(pool, inserted_private_message.id).await?;

    let private_message_update_form = PrivateMessageUpdateForm {
      content: Some("A test private message".into()),
      ..Default::default()
    };
    let updated_private_message = PrivateMessage::update(
      pool,
      inserted_private_message.id,
      &private_message_update_form,
    )
    .await?;

    let deleted_private_message = PrivateMessage::update(
      pool,
      inserted_private_message.id,
      &PrivateMessageUpdateForm {
        deleted: Some(true),
        ..Default::default()
      },
    )
    .await?;
    Person::delete(pool, inserted_creator.id).await?;
    Person::delete(pool, inserted_recipient.id).await?;
    Instance::delete(pool, inserted_instance.id).await?;

    assert_eq!(expected_private_message, read_private_message);
    assert_eq!(expected_private_message, updated_private_message);
    assert_eq!(expected_private_message, inserted_private_message);
    assert!(deleted_private_message.deleted);

    Ok(())
  }
}
