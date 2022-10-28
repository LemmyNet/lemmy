use crate::{
  newtypes::{DbUrl, PersonId, PrivateMessageId},
  source::private_message::*,
  traits::{Crud, DeleteableOrRemoveable},
};
use diesel::{dsl::*, result::Error, *};
use lemmy_utils::error::LemmyError;
use url::Url;

impl Crud for PrivateMessage {
  type InsertForm = PrivateMessageInsertForm;
  type UpdateForm = PrivateMessageUpdateForm;
  type IdType = PrivateMessageId;
  fn read(conn: &mut PgConnection, private_message_id: PrivateMessageId) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    private_message.find(private_message_id).first::<Self>(conn)
  }

  fn create(conn: &mut PgConnection, form: &Self::InsertForm) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    insert_into(private_message)
      .values(form)
      .on_conflict(ap_id)
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &mut PgConnection,
    private_message_id: PrivateMessageId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(form)
      .get_result::<Self>(conn)
  }
  fn delete(conn: &mut PgConnection, pm_id: Self::IdType) -> Result<usize, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::delete(private_message.find(pm_id)).execute(conn)
  }
}

impl PrivateMessage {
  pub fn mark_all_as_read(
    conn: &mut PgConnection,
    for_recipient_id: PersonId,
  ) -> Result<Vec<PrivateMessage>, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(
      private_message
        .filter(recipient_id.eq(for_recipient_id))
        .filter(read.eq(false)),
    )
    .set(read.eq(true))
    .get_results::<Self>(conn)
  }

  pub fn read_from_apub_id(
    conn: &mut PgConnection,
    object_id: Url,
  ) -> Result<Option<Self>, LemmyError> {
    use crate::schema::private_message::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      private_message
        .filter(ap_id.eq(object_id))
        .first::<PrivateMessage>(conn)
        .ok()
        .map(Into::into),
    )
  }
}

impl DeleteableOrRemoveable for PrivateMessage {
  fn blank_out_deleted_or_removed_info(mut self) -> Self {
    self.content = "".into();
    self
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    source::{instance::Instance, person::*, private_message::*},
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = &mut establish_unpooled_connection();

    let inserted_instance = Instance::create(conn, "my_domain.tld").unwrap();

    let creator_form = PersonInsertForm::builder()
      .name("creator_pm".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_creator = Person::create(conn, &creator_form).unwrap();

    let recipient_form = PersonInsertForm::builder()
      .name("recipient_pm".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_recipient = Person::create(conn, &recipient_form).unwrap();

    let private_message_form = PrivateMessageInsertForm::builder()
      .content("A test private message".into())
      .creator_id(inserted_creator.id)
      .recipient_id(inserted_recipient.id)
      .build();

    let inserted_private_message = PrivateMessage::create(conn, &private_message_form).unwrap();

    let expected_private_message = PrivateMessage {
      id: inserted_private_message.id,
      content: "A test private message".into(),
      creator_id: inserted_creator.id,
      recipient_id: inserted_recipient.id,
      deleted: false,
      read: false,
      updated: None,
      published: inserted_private_message.published,
      ap_id: inserted_private_message.ap_id.to_owned(),
      local: true,
    };

    let read_private_message = PrivateMessage::read(conn, inserted_private_message.id).unwrap();

    let private_message_update_form = PrivateMessageUpdateForm::builder()
      .content(Some("A test private message".into()))
      .build();
    let updated_private_message = PrivateMessage::update(
      conn,
      inserted_private_message.id,
      &private_message_update_form,
    )
    .unwrap();

    let deleted_private_message = PrivateMessage::update(
      conn,
      inserted_private_message.id,
      &PrivateMessageUpdateForm::builder()
        .deleted(Some(true))
        .build(),
    )
    .unwrap();
    let marked_read_private_message = PrivateMessage::update(
      conn,
      inserted_private_message.id,
      &PrivateMessageUpdateForm::builder().read(Some(true)).build(),
    )
    .unwrap();
    Person::delete(conn, inserted_creator.id).unwrap();
    Person::delete(conn, inserted_recipient.id).unwrap();
    Instance::delete(conn, inserted_instance.id).unwrap();

    assert_eq!(expected_private_message, read_private_message);
    assert_eq!(expected_private_message, updated_private_message);
    assert_eq!(expected_private_message, inserted_private_message);
    assert!(deleted_private_message.deleted);
    assert!(marked_read_private_message.read);
  }
}
