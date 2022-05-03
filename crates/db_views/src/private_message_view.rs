use crate::structs::PrivateMessageView;
use diesel::{pg::Pg, result::Error, *};
use lemmy_db_schema::{
  newtypes::{PersonId, PrivateMessageId},
  schema::{person, person_alias_1, private_message},
  source::{
    person::{Person, PersonAlias1, PersonSafe, PersonSafeAlias1},
    private_message::PrivateMessage,
  },
  traits::{MaybeOptional, ToSafe, ViewToVec},
  utils::limit_and_offset,
};
use tracing::debug;

type PrivateMessageViewTuple = (PrivateMessage, PersonSafe, PersonSafeAlias1);

impl PrivateMessageView {
  pub fn read(conn: &PgConnection, private_message_id: PrivateMessageId) -> Result<Self, Error> {
    let (private_message, creator, recipient) = private_message::table
      .find(private_message_id)
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(private_message::recipient_id.eq(person_alias_1::id)))
      .order_by(private_message::published.desc())
      .select((
        private_message::all_columns,
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .first::<PrivateMessageViewTuple>(conn)?;

    Ok(PrivateMessageView {
      private_message,
      creator,
      recipient,
    })
  }

  /// Gets the number of unread messages
  pub fn get_unread_messages(conn: &PgConnection, my_person_id: PersonId) -> Result<i64, Error> {
    use diesel::dsl::*;
    private_message::table
      .filter(private_message::read.eq(false))
      .filter(private_message::recipient_id.eq(my_person_id))
      .filter(private_message::deleted.eq(false))
      .select(count(private_message::id))
      .first::<i64>(conn)
  }
}

pub struct PrivateMessageQueryBuilder<'a> {
  conn: &'a PgConnection,
  recipient_id: PersonId,
  unread_only: Option<bool>,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PrivateMessageQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, recipient_id: PersonId) -> Self {
    PrivateMessageQueryBuilder {
      conn,
      recipient_id,
      unread_only: None,
      page: None,
      limit: None,
    }
  }

  pub fn unread_only<T: MaybeOptional<bool>>(mut self, unread_only: T) -> Self {
    self.unread_only = unread_only.get_optional();
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
    let mut query = private_message::table
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(person_alias_1::table.on(private_message::recipient_id.eq(person_alias_1::id)))
      .select((
        private_message::all_columns,
        Person::safe_columns_tuple(),
        PersonAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    // If its unread, I only want the ones to me
    if self.unread_only.unwrap_or(false) {
      query = query
        .filter(private_message::read.eq(false))
        .filter(private_message::recipient_id.eq(self.recipient_id));
    }
    // Otherwise, I want the ALL view to show both sent and received
    else {
      query = query.filter(
        private_message::recipient_id
          .eq(self.recipient_id)
          .or(private_message::creator_id.eq(self.recipient_id)),
      )
    }

    let (limit, offset) = limit_and_offset(self.page, self.limit);

    query = query
      .filter(private_message::deleted.eq(false))
      .limit(limit)
      .offset(offset)
      .order_by(private_message::published.desc());

    debug!(
      "Private Message View Query: {:?}",
      debug_query::<Pg, _>(&query)
    );

    let res = query.load::<PrivateMessageViewTuple>(self.conn)?;

    Ok(PrivateMessageView::from_tuple_to_vec(res))
  }
}

impl ViewToVec for PrivateMessageView {
  type DbTuple = PrivateMessageViewTuple;
  fn from_tuple_to_vec(items: Vec<Self::DbTuple>) -> Vec<Self> {
    items
      .iter()
      .map(|a| Self {
        private_message: a.0.to_owned(),
        creator: a.1.to_owned(),
        recipient: a.2.to_owned(),
      })
      .collect::<Vec<Self>>()
  }
}
