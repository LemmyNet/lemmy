use super::*;
use crate::apub::{make_apub_endpoint, EndpointType};
use crate::schema::private_message;

#[derive(Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize)]
#[table_name = "private_message"]
pub struct PrivateMessage {
  pub id: i32,
  pub creator_id: i32,
  pub recipient_id: i32,
  pub content: String,
  pub deleted: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: String,
  pub local: bool,
}

#[derive(Insertable, AsChangeset, Clone)]
#[table_name = "private_message"]
pub struct PrivateMessageForm {
  pub creator_id: i32,
  pub recipient_id: i32,
  pub content: String,
  pub deleted: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: String,
  pub local: bool,
}

impl Crud<PrivateMessageForm> for PrivateMessage {
  fn read(conn: &PgConnection, private_message_id: i32) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    private_message.find(private_message_id).first::<Self>(conn)
  }

  fn delete(conn: &PgConnection, private_message_id: i32) -> Result<usize, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::delete(private_message.find(private_message_id)).execute(conn)
  }

  fn create(conn: &PgConnection, private_message_form: &PrivateMessageForm) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    insert_into(private_message)
      .values(private_message_form)
      .get_result::<Self>(conn)
  }

  fn update(
    conn: &PgConnection,
    private_message_id: i32,
    private_message_form: &PrivateMessageForm,
  ) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    diesel::update(private_message.find(private_message_id))
      .set(private_message_form)
      .get_result::<Self>(conn)
  }
}

impl PrivateMessage {
  pub fn update_ap_id(conn: &PgConnection, private_message_id: i32) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;

    let apid = make_apub_endpoint(
      EndpointType::PrivateMessage,
      &private_message_id.to_string(),
    )
    .to_string();
    diesel::update(private_message.find(private_message_id))
      .set(ap_id.eq(apid))
      .get_result::<Self>(conn)
  }

  pub fn read_from_apub_id(conn: &PgConnection, object_id: &str) -> Result<Self, Error> {
    use crate::schema::private_message::dsl::*;
    private_message
      .filter(ap_id.eq(object_id))
      .first::<Self>(conn)
  }
}

#[cfg(test)]
mod tests {
  use super::super::user::*;
  use super::*;
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
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: "changeme".into(),
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
      admin: false,
      banned: false,
      updated: None,
      show_nsfw: false,
      theme: "darkly".into(),
      default_sort_type: SortType::Hot as i16,
      default_listing_type: ListingType::Subscribed as i16,
      lang: "browser".into(),
      show_avatars: true,
      send_notifications_to_email: false,
      actor_id: "changeme".into(),
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
      ap_id: "changeme".into(),
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
      ap_id: "changeme".into(),
      local: true,
    };

    let read_private_message = PrivateMessage::read(&conn, inserted_private_message.id).unwrap();
    let updated_private_message =
      PrivateMessage::update(&conn, inserted_private_message.id, &private_message_form).unwrap();
    let num_deleted = PrivateMessage::delete(&conn, inserted_private_message.id).unwrap();
    User_::delete(&conn, inserted_creator.id).unwrap();
    User_::delete(&conn, inserted_recipient.id).unwrap();

    assert_eq!(expected_private_message, read_private_message);
    assert_eq!(expected_private_message, updated_private_message);
    assert_eq!(expected_private_message, inserted_private_message);
    assert_eq!(1, num_deleted);
  }
}
