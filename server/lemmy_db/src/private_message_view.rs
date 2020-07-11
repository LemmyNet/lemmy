use crate::{limit_and_offset, MaybeOptional};
use diesel::{pg::Pg, result::Error, *};
use serde::{Deserialize, Serialize};

// The faked schema since diesel doesn't do views
table! {
  private_message_view (id) {
    id -> Int4,
    creator_id -> Int4,
    recipient_id -> Int4,
    content -> Text,
    deleted -> Bool,
    read -> Bool,
    published -> Timestamp,
    updated -> Nullable<Timestamp>,
    ap_id -> Text,
    local -> Bool,
    creator_name -> Varchar,
    creator_avatar -> Nullable<Text>,
    creator_actor_id -> Text,
    creator_local -> Bool,
    recipient_name -> Varchar,
    recipient_avatar -> Nullable<Text>,
    recipient_actor_id -> Text,
    recipient_local -> Bool,
  }
}

#[derive(
  Queryable, Identifiable, PartialEq, Debug, Serialize, Deserialize, QueryableByName, Clone,
)]
#[table_name = "private_message_view"]
pub struct PrivateMessageView {
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
  pub creator_name: String,
  pub creator_avatar: Option<String>,
  pub creator_actor_id: String,
  pub creator_local: bool,
  pub recipient_name: String,
  pub recipient_avatar: Option<String>,
  pub recipient_actor_id: String,
  pub recipient_local: bool,
}

pub struct PrivateMessageQueryBuilder<'a> {
  conn: &'a PgConnection,
  query: super::private_message_view::private_message_view::BoxedQuery<'a, Pg>,
  for_recipient_id: i32,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PrivateMessageQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, for_recipient_id: i32) -> Self {
    use super::private_message_view::private_message_view::dsl::*;

    let query = private_message_view.into_boxed();

    PrivateMessageQueryBuilder {
      conn,
      query,
      for_recipient_id,
      unread_only: false,
      page: None,
      limit: None,
    }
  }

  pub fn unread_only(mut self, unread_only: bool) -> Self {
    self.unread_only = unread_only;
    self
  }

  pub fn page<T: MaybeOptional<i64>>(mut self, page: T) -> Self {
    self.page = page.get_optional();
    self
  }

  pub fn limit<T: MaybeOptional<i64>>(mut self, limit: T) -> Self {
    self.limit = limit.get_optional();
    self
  }

  pub fn list(self) -> Result<Vec<PrivateMessageView>, Error> {
    use super::private_message_view::private_message_view::dsl::*;

    let mut query = self.query.filter(deleted.eq(false));

    // If its unread, I only want the ones to me
    if self.unread_only {
      query = query
        .filter(read.eq(false))
        .filter(recipient_id.eq(self.for_recipient_id));
    }
    // Otherwise, I want the ALL view to show both sent and received
    else {
      query = query.filter(
        recipient_id
          .eq(self.for_recipient_id)
          .or(creator_id.eq(self.for_recipient_id)),
      )
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    query
      .limit(limit)
      .offset(offset)
      .order_by(published.desc())
      .load::<PrivateMessageView>(self.conn)
  }
}

impl PrivateMessageView {
  pub fn read(conn: &PgConnection, from_private_message_id: i32) -> Result<Self, Error> {
    use super::private_message_view::private_message_view::dsl::*;

    let mut query = private_message_view.into_boxed();

    query = query
      .filter(id.eq(from_private_message_id))
      .order_by(published.desc());

    query.first::<Self>(conn)
  }
}
