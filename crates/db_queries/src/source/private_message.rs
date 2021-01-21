use crate::{ApubObject, Crud};
use diesel::{dsl::*, result::Error, *};
use lemmy_db_schema::{naive_now, source::private_message::*};

impl Crud<PrivateMessageForm> for PrivateMessage {
  fn read(conn: &PgConnection, private_message_id: i32) -> Result<Self, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    private_message.find(private_message_id).first::<Self>(conn)
  }

  fn create(conn: &PgConnection, private_message_form: &PrivateMessageForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    insert_into(private_message)
      .values(private_message_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    private_message_id: i32,
    private_message_form: &PrivateMessageForm,
  ) -> Result<Self, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(private_message_form)
      .get_result::<Self>(conn)
  }
}

impl ApubObject<PrivateMessageForm> for PrivateMessage {
  fn read_from_apub_id(conn: &PgConnection, object_id: &str) -> Result<Self, Error>
  where
    Self: Sized,
  {
    use lemmy_db_schema::schema::private_message::dsl::*;
    private_message
      .filter(ap_id.eq(object_id))
      .first::<Self>(conn)
  }

  fn upsert(conn: &PgConnection, private_message_form: &PrivateMessageForm) -> Result<Self, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    insert_into(private_message)
      .values(private_message_form)
      .on_conflict(ap_id)
      .do_update()
      .set(private_message_form)
      .get_result::<Self>(conn)
  }
}

pub trait PrivateMessage_ {
  fn update_ap_id(
    conn: &PgConnection,
    private_message_id: i32,
    apub_id: String,
  ) -> Result<PrivateMessage, Error>;
  fn update_content(
    conn: &PgConnection,
    private_message_id: i32,
    new_content: &str,
  ) -> Result<PrivateMessage, Error>;
  fn update_deleted(
    conn: &PgConnection,
    private_message_id: i32,
    new_deleted: bool,
  ) -> Result<PrivateMessage, Error>;
  fn update_read(
    conn: &PgConnection,
    private_message_id: i32,
    new_read: bool,
  ) -> Result<PrivateMessage, Error>;
  fn mark_all_as_read(
    conn: &PgConnection,
    for_recipient_id: i32,
  ) -> Result<Vec<PrivateMessage>, Error>;
}

impl PrivateMessage_ for PrivateMessage {
  fn update_ap_id(
    conn: &PgConnection,
    private_message_id: i32,
    apub_id: String,
  ) -> Result<PrivateMessage, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;

    diesel::update(private_message.find(private_message_id))
      .set(ap_id.eq(apub_id))
      .get_result::<Self>(conn)
  }

  fn update_content(
    conn: &PgConnection,
    private_message_id: i32,
    new_content: &str,
  ) -> Result<PrivateMessage, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set((content.eq(new_content), updated.eq(naive_now())))
      .get_result::<Self>(conn)
  }

  fn update_deleted(
    conn: &PgConnection,
    private_message_id: i32,
    new_deleted: bool,
  ) -> Result<PrivateMessage, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(deleted.eq(new_deleted))
      .get_result::<Self>(conn)
  }

  fn update_read(
    conn: &PgConnection,
    private_message_id: i32,
    new_read: bool,
  ) -> Result<PrivateMessage, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(read.eq(new_read))
      .get_result::<Self>(conn)
  }

  fn mark_all_as_read(
    conn: &PgConnection,
    for_recipient_id: i32,
  ) -> Result<Vec<PrivateMessage>, Error> {
    use lemmy_db_schema::schema::private_message::dsl::*;
    diesel::update(
      private_message
        .filter(recipient_id.eq(for_recipient_id))
        .filter(read.eq(false)),
    )
    .set(read.eq(true))
    .get_results::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    establish_unpooled_connection,
    source::private_message::PrivateMessage_,
    Crud,
    ListingType,
    SortType,
  };
  use lemmy_db_schema::source::{private_message::*, user::*};

  #[test]
  fn test_crud() {
    let conn = establish_unpooled_connection();

    let creator_form = UserForm {
      name: "creator_pm".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
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

    let recipient_form = UserForm {
      name: "recipient_pm".into(),
      preferred_username: None,
      password_encrypted: "nope".into(),
      email: None,
      matrix_user_id: None,
      avatar: None,
      banner: None,
      admin: false,
      banned: Some(false),
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

    let inserted_recipient = User_::create(&conn, &recipient_form).unwrap();

    let private_message_form = PrivateMessageForm {
      content: "A test private message".into(),
      creator_id: inserted_creator.id,
      recipient_id: inserted_recipient.id,
      deleted: None,
      read: None,
      published: None,
      updated: None,
      ap_id: None,
      local: true,
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
    User_::delete(&conn, inserted_creator.id).unwrap();
    User_::delete(&conn, inserted_recipient.id).unwrap();

    assert_eq!(expected_private_message, read_private_message);
    assert_eq!(expected_private_message, updated_private_message);
    assert_eq!(expected_private_message, inserted_private_message);
    assert!(deleted_private_message.deleted);
    assert!(marked_read_private_message.read);
  }
}
