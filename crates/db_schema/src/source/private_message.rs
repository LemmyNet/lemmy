use crate::{schema::private_message, DbUrl, PersonId, PrivateMessageId};
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use lemmy_apub_lib::traits::ApubObject;
use lemmy_utils::LemmyError;
use serde::Serialize;
use url::Url;

#[derive(Clone, Queryable, Associations, Identifiable, PartialEq, Debug, Serialize)]
#[table_name = "private_message"]
pub struct PrivateMessage {
  pub id: PrivateMessageId,
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  pub deleted: bool,
  pub read: bool,
  pub published: chrono::NaiveDateTime,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: DbUrl,
  pub local: bool,
}

#[derive(Insertable, AsChangeset, Default)]
#[table_name = "private_message"]
pub struct PrivateMessageForm {
  pub creator_id: PersonId,
  pub recipient_id: PersonId,
  pub content: String,
  pub deleted: Option<bool>,
  pub read: Option<bool>,
  pub published: Option<chrono::NaiveDateTime>,
  pub updated: Option<chrono::NaiveDateTime>,
  pub ap_id: Option<DbUrl>,
  pub local: Option<bool>,
}

impl ApubObject for PrivateMessage {
  type DataType = PgConnection;

  fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
    None
  }

  fn read_from_apub_id(conn: &PgConnection, object_id: Url) -> Result<Option<Self>, LemmyError> {
    use crate::schema::private_message::dsl::*;
    let object_id: DbUrl = object_id.into();
    Ok(
      private_message
        .filter(ap_id.eq(object_id))
        .first::<Self>(conn)
        .ok(),
    )
  }

  fn delete(self, _conn: &PgConnection) -> Result<(), LemmyError> {
    // do nothing, because pm can't be fetched over http
    unimplemented!()
  }
}
