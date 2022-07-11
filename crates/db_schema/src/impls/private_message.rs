use crate::{
  newtypes::{DbUrl, PersonId, PrivateMessageId},
  source::private_message::*,
  traits::{Crud, DeleteableOrRemoveable},
  utils::naive_now,
};
use diesel::{dsl::*, result::Error, *};
use lemmy_utils::error::LemmyError;
use url::Url;

impl Crud for PrivateMessage {
  type Form = PrivateMessageForm;
  type IdType = PrivateMessageId;
  fn read(conn: &PgConnection, private_message_id: PrivateMessageId) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    private_message.find(private_message_id).first::<Self>(conn)
  }

  fn create(conn: &PgConnection, private_message_form: &PrivateMessageForm) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    insert_into(private_message)
      .values(private_message_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    private_message_id: PrivateMessageId,
    private_message_form: &PrivateMessageForm,
  ) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(private_message_form)
      .get_result::<Self>(conn)
  }
  fn delete(conn: &PgConnection, pm_id: Self::IdType) -> Result<usize, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::delete(private_message.find(pm_id)).execute(conn)
  }
}

impl PrivateMessage {
  pub fn update_ap_id(
    conn: &PgConnection,
    private_message_id: PrivateMessageId,
    apub_id: DbUrl,
  ) -> Result<PrivateMessage, Error> {
    use crate::schema::private_message::dsl::*;

    diesel::update(private_message.find(private_message_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  pub fn update_content(
    conn: &PgConnection,
    private_message_id: PrivateMessageId,
    new_content: &str,
  ) -> Result<PrivateMessage, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set((content.eq(new_content), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  pub fn update_deleted(
    conn: &PgConnection,
    private_message_id: PrivateMessageId,
    new_deleted: bool,
  ) -> Result<PrivateMessage, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(deleted.eq(new_deleted))
      .get_result::<Self>(conn)
  }

  pub fn update_read(
    conn: &PgConnection,
    private_message_id: PrivateMessageId,
    new_read: bool,
  ) -> Result<PrivateMessage, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(read.eq(new_read))
      .get_result::<Self>(conn)
  }

  pub fn mark_all_as_read(
    conn: &PgConnection,
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

  pub fn upsert(
    conn: &PgConnection,
    private_message_form: &PrivateMessageForm,
  ) -> Result<PrivateMessage, Error> {
    use crate::schema::private_message::dsl::*;
    insert_into(private_message)
      .values(private_message_form)
      .on_conflict(ap_id)
      .do_update()
      .set(private_message_form)
      .get_result::<Self>(conn)
  }

  pub fn read_from_apub_id(
    conn: &PgConnection,
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
    source::{person::*, private_message::*},
    traits::Crud,
    utils::establish_unpooled_connection,
  };
  use serial_test::serial;

  #[test]
  #[serial]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let creator_form = PersonForm {
      name: "creator_pm".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_creator = Person::create(&conn, &creator_form).unwrap();

    let recipient_form = PersonForm {
      name: "recipient_pm".into(),
      public_key: Some("pubkey".to_string()),
      ..PersonForm::default()
    };

    let inserted_recipient = Person::create(&conn, &recipient_form).unwrap();

    let private_message_form = PrivateMessageForm {
      content: "A test private message".into(),
      creator_id: inserted_creator.id,
      recipient_id: inserted_recipient.id,
      ..PrivateMessageForm::default()
    };

    let inserted_private_message = PrivateMessage::create(&conn, &private_message_form).unwrap();

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

    let read_private_message = PrivateMessage::read(&conn, inserted_private_message.id).unwrap();
    let updated_private_message =
      PrivateMessage::update(&conn, inserted_private_message.id, &private_message_form).unwrap();
    let deleted_private_message =
      PrivateMessage::update_deleted(&conn, inserted_private_message.id, true).unwrap();
    let marked_read_private_message =
      PrivateMessage::update_read(&conn, inserted_private_message.id, true).unwrap();
    Person::delete(&conn, inserted_creator.id).unwrap();
    Person::delete(&conn, inserted_recipient.id).unwrap();

    assert_eq!(expected_private_message, read_private_message);
    assert_eq!(expected_private_message, updated_private_message);
    assert_eq!(expected_private_message, inserted_private_message);
    assert!(deleted_private_message.deleted);
    assert!(marked_read_private_message.read);
  }
}
