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
    if options.unread_only.unwrap_or(false) {
      query = query.filter(private_message::read.eq(false));
      if let Some(i) = options.from {
        query = query.filter(private_message::creator_id.eq(i))
      }
      query = query.filter(private_message::recipient_id.eq(recipient_id));
    }
    // Otherwise, I want the ALL view to show both sent and received
    else {
      query = query.filter(
        private_message::recipient_id
          .eq(recipient_id)
          .or(private_message::creator_id.eq(recipient_id)),
      );
      if let Some(i) = options.from {
        query = query.filter(
          private_message::creator_id
            .eq(i)
            .or(private_message::recipient_id.eq(i)),
        )
      }
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
  pub unread_only: Option<bool>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
  pub from: Option<PersonId>,
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

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::private_message_view::PrivateMessageQuery;
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
    },
    traits::Crud,
    utils::build_db_pool_for_tests,
  };
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let message_content = String::from("");
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let timmy_person_form = PersonInsertForm::builder()
      .name("timmy_rav".into())
      .admin(Some(true))
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_timmy_person = Person::create(pool, &timmy_person_form).await.unwrap();

    let timmy_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_timmy_person.id)
      .password_encrypted("nada".to_string())
      .build();

    let _inserted_timmy_local_user = LocalUser::create(pool, &timmy_local_user_form)
      .await
      .unwrap();

    let sara_person_form = PersonInsertForm::builder()
      .name("sara_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_sara_person = Person::create(pool, &sara_person_form).await.unwrap();

    let sara_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_sara_person.id)
      .password_encrypted("nada".to_string())
      .build();

    let _inserted_sara_local_user = LocalUser::create(pool, &sara_local_user_form)
      .await
      .unwrap();

    let jess_person_form = PersonInsertForm::builder()
      .name("jess_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_jess_person = Person::create(pool, &jess_person_form).await.unwrap();

    let jess_local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_jess_person.id)
      .password_encrypted("nada".to_string())
      .build();

    let _inserted_jess_local_user = LocalUser::create(pool, &jess_local_user_form)
      .await
      .unwrap();

    let sara_timmy_message_form = PrivateMessageInsertForm::builder()
      .creator_id(inserted_sara_person.id)
      .recipient_id(inserted_timmy_person.id)
      .content(message_content.clone())
      .build();
    let _inserted_sara_timmy_message_form = PrivateMessage::create(pool, &sara_timmy_message_form)
      .await
      .unwrap();

    let sara_jess_message_form = PrivateMessageInsertForm::builder()
      .creator_id(inserted_sara_person.id)
      .recipient_id(inserted_jess_person.id)
      .content(message_content.clone())
      .build();
    let _inserted_sara_jess_message_form = PrivateMessage::create(pool, &sara_jess_message_form)
      .await
      .unwrap();

    let timmy_sara_message_form = PrivateMessageInsertForm::builder()
      .creator_id(inserted_timmy_person.id)
      .recipient_id(inserted_sara_person.id)
      .content(message_content.clone())
      .build();
    let _inserted_timmy_sara_message_form = PrivateMessage::create(pool, &timmy_sara_message_form)
      .await
      .unwrap();

    let jess_timmy_message_form = PrivateMessageInsertForm::builder()
      .creator_id(inserted_jess_person.id)
      .recipient_id(inserted_timmy_person.id)
      .content(message_content.clone())
      .build();
    let _inserted_jess_timmy_message_form = PrivateMessage::create(pool, &jess_timmy_message_form)
      .await
      .unwrap();

    let timmy_messages = PrivateMessageQuery {
      unread_only: Some(false),
      from: Option::None,
      page: Some(1),
      limit: Some(20),
    }
    .list(pool, inserted_timmy_person.id)
    .await
    .unwrap();

    assert_eq!(timmy_messages.len(), 3);

    let timmy_unread_messages = PrivateMessageQuery {
      unread_only: Some(true),
      from: Option::None,
      page: Some(1),
      limit: Some(20),
    }
    .list(pool, inserted_timmy_person.id)
    .await
    .unwrap();

    assert_eq!(timmy_unread_messages.len(), 2);

    let timmy_sara_messages = PrivateMessageQuery {
      unread_only: Some(false),
      from: Some(inserted_sara_person.id),
      page: Some(1),
      limit: Some(20),
    }
    .list(pool, inserted_timmy_person.id)
    .await
    .unwrap();

    assert_eq!(timmy_sara_messages.len(), 2);

    let timmy_sara_unread_messages = PrivateMessageQuery {
      unread_only: Some(true),
      from: Some(inserted_sara_person.id),
      page: Some(1),
      limit: Some(20),
    }
    .list(pool, inserted_timmy_person.id)
    .await
    .unwrap();

    assert_eq!(timmy_sara_unread_messages.len(), 1);
  }
}
