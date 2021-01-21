use diesel::{pg::Pg, result::Error, *};
use lemmy_db_queries::{limit_and_offset, MaybeOptional, ToSafe, ViewToVec};
use lemmy_db_schema::{
  schema::{private_message, user_, user_alias_1},
  source::{
    private_message::PrivateMessage,
    user::{UserAlias1, UserSafe, UserSafeAlias1, User_},
  },
};
use log::debug;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Clone)]
pub struct PrivateMessageView {
  pub private_message: PrivateMessage,
  pub creator: UserSafe,
  pub recipient: UserSafeAlias1,
}

type PrivateMessageViewTuple = (PrivateMessage, UserSafe, UserSafeAlias1);

impl PrivateMessageView {
  pub fn read(conn: &PgConnection, private_message_id: i32) -> Result<Self, Error> {
    let (private_message, creator, recipient) = private_message::table
      .find(private_message_id)
      .inner_join(user_::table.on(private_message::creator_id.eq(user_::id)))
      .inner_join(user_alias_1::table.on(private_message::recipient_id.eq(user_alias_1::id)))
      .order_by(private_message::published.desc())
      .select((
        private_message::all_columns,
        User_::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
      ))
      .first::<PrivateMessageViewTuple>(conn)?;

    Ok(PrivateMessageView {
      private_message,
      creator,
      recipient,
    })
  }
}

pub struct PrivateMessageQueryBuilder<'a> {
  conn: &'a PgConnection,
  recipient_id: i32,
  unread_only: bool,
  page: Option<i64>,
  limit: Option<i64>,
}

impl<'a> PrivateMessageQueryBuilder<'a> {
  pub fn create(conn: &'a PgConnection, recipient_id: i32) -> Self {
    PrivateMessageQueryBuilder {
      conn,
      recipient_id,
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
    let mut query = private_message::table
      .inner_join(user_::table.on(private_message::creator_id.eq(user_::id)))
      .inner_join(user_alias_1::table.on(private_message::recipient_id.eq(user_alias_1::id)))
      .select((
        private_message::all_columns,
        User_::safe_columns_tuple(),
        UserAlias1::safe_columns_tuple(),
      ))
      .into_boxed();

    // If its unread, I only want the ones to me
    if self.unread_only {
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
