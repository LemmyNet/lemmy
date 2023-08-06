use crate::structs::PrivateMessageView;
use diesel::{
  debug_query,
  pg::Pg,
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  JoinOnDsl,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  aliases,
  newtypes::{PersonId, PrivateMessageId},
  schema::{person, private_message},
  source::{person::Person, private_message::PrivateMessage},
  traits::JoinView,
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};
use tracing::debug;

type PrivateMessageViewTuple = (PrivateMessage, Person, Person);

fn queries<'a>() -> Queries<
  impl ReadFn<'a, PrivateMessageView, PrivateMessageId>,
  impl ListFn<'a, PrivateMessageView, (PrivateMessageQuery, PersonId)>,
> {
  let all_joins = |query: private_message::BoxedQuery<'a, Pg>| {
    query
      .inner_join(person::table.on(private_message::creator_id.eq(person::id)))
      .inner_join(
        aliases::person1.on(private_message::recipient_id.eq(aliases::person1.field(person::id))),
      )
  };

  let selection = (
    private_message::all_columns,
    person::all_columns,
    aliases::person1.fields(person::all_columns),
  );

  let read = move |mut conn: DbConn<'a>, private_message_id: PrivateMessageId| async move {
    all_joins(private_message::table.find(private_message_id).into_boxed())
      .order_by(private_message::published.desc())
      .select(selection)
      .first::<PrivateMessageViewTuple>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>,
                   (options, recipient_id): (PrivateMessageQuery, PersonId)| async move {
    let mut query = all_joins(private_message::table.into_boxed()).select(selection);

    // If its unread, I only want the ones to me
    if options.unread_only {
      query = query
        .filter(private_message::read.eq(false))
        .filter(private_message::recipient_id.eq(recipient_id));
    }
    // Otherwise, I want the ALL view to show both sent and received
    else {
      query = query.filter(
        private_message::recipient_id
          .eq(recipient_id)
          .or(private_message::creator_id.eq(recipient_id)),
      )
    }

    let (limit, offset) = limit_and_offset(options.page, options.limit)?;

    query = query
      .filter(private_message::deleted.eq(false))
      .limit(limit)
      .offset(offset)
      .order_by(private_message::published.desc());

    debug!(
      "Private Message View Query: {:?}",
      debug_query::<Pg, _>(&query)
    );

    query.load::<PrivateMessageViewTuple>(&mut conn).await
  };

  Queries::new(read, list)
}

impl PrivateMessageView {
  pub async fn read(
    pool: &mut DbPool<'_>,
    private_message_id: PrivateMessageId,
  ) -> Result<Self, Error> {
    queries().read(pool, private_message_id).await
  }

  /// Gets the number of unread messages
  pub async fn get_unread_messages(
    pool: &mut DbPool<'_>,
    my_person_id: PersonId,
  ) -> Result<i64, Error> {
    use diesel::dsl::count;
    let conn = &mut get_conn(pool).await?;
    private_message::table
      .filter(private_message::read.eq(false))
      .filter(private_message::recipient_id.eq(my_person_id))
      .filter(private_message::deleted.eq(false))
      .select(count(private_message::id))
      .first::<i64>(conn)
      .await
  }
}

#[derive(Default)]
pub struct PrivateMessageQuery {
  pub unread_only: bool,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}

impl PrivateMessageQuery {
  pub async fn list(
    self,
    pool: &mut DbPool<'_>,
    recipient_id: PersonId,
  ) -> Result<Vec<PrivateMessageView>, Error> {
    queries().list(pool, (self, recipient_id)).await
  }
}

impl JoinView for PrivateMessageView {
  type JoinTuple = PrivateMessageViewTuple;
  fn from_tuple(a: Self::JoinTuple) -> Self {
    Self {
      private_message: a.0,
      creator: a.1,
      recipient: a.2,
    }
  }
}
