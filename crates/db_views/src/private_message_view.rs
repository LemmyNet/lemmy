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
  schema::{person, person_block, private_message},
  utils::{get_conn, limit_and_offset, DbConn, DbPool, ListFn, Queries, ReadFn},
};
use tracing::debug;

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
      .left_join(
        person_block::table.on(
          private_message::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(aliases::person1.field(person::id))),
        ),
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
      .first::<PrivateMessageView>(&mut conn)
      .await
  };

  let list = move |mut conn: DbConn<'a>,
                   (options, recipient_id): (PrivateMessageQuery, PersonId)| async move {
    let mut query = all_joins(private_message::table.into_boxed())
      .select(selection)
      // Dont show replies from blocked users
      .filter(person_block::person_id.is_null());

    // If its unread, I only want the ones to me
    if options.unread_only {
      query = query.filter(private_message::read.eq(false));
      if let Some(i) = options.creator_id {
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
      if let Some(i) = options.creator_id {
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

    query.load::<PrivateMessageView>(&mut conn).await
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
      .left_join(
        person_block::table.on(
          private_message::creator_id
            .eq(person_block::target_id)
            .and(person_block::person_id.eq(my_person_id)),
        ),
      )
      // Dont count replies from blocked users
      .filter(person_block::person_id.is_null())
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
  pub creator_id: Option<PersonId>,
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

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::indexing_slicing)]

  use crate::{private_message_view::PrivateMessageQuery, structs::PrivateMessageView};
  use lemmy_db_schema::{
    assert_length,
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
      person_block::{PersonBlock, PersonBlockForm},
      private_message::{PrivateMessage, PrivateMessageInsertForm},
    },
    traits::{Blockable, Crud},
    utils::build_db_pool_for_tests,
  };
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  #[tokio::test]
  #[serial]
  async fn test_crud() {
    let message_content = String::new();
    let pool = &build_db_pool_for_tests().await;
    let pool = &mut pool.into();

    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let timmy_form = PersonInsertForm::builder()
      .name("timmy_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();

    let timmy = Person::create(pool, &timmy_form).await.unwrap();

    let sara_form = PersonInsertForm::builder()
      .name("sara_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();

    let sara = Person::create(pool, &sara_form).await.unwrap();

    let jess_form = PersonInsertForm::builder()
      .name("jess_rav".into())
      .public_key("pubkey".to_string())
      .instance_id(instance.id)
      .build();

    let jess = Person::create(pool, &jess_form).await.unwrap();

    let sara_timmy_message_form = PrivateMessageInsertForm::builder()
      .creator_id(sara.id)
      .recipient_id(timmy.id)
      .content(message_content.clone())
      .build();
    PrivateMessage::create(pool, &sara_timmy_message_form)
      .await
      .unwrap();

    let sara_jess_message_form = PrivateMessageInsertForm::builder()
      .creator_id(sara.id)
      .recipient_id(jess.id)
      .content(message_content.clone())
      .build();
    PrivateMessage::create(pool, &sara_jess_message_form)
      .await
      .unwrap();

    let timmy_sara_message_form = PrivateMessageInsertForm::builder()
      .creator_id(timmy.id)
      .recipient_id(sara.id)
      .content(message_content.clone())
      .build();
    PrivateMessage::create(pool, &timmy_sara_message_form)
      .await
      .unwrap();

    let jess_timmy_message_form = PrivateMessageInsertForm::builder()
      .creator_id(jess.id)
      .recipient_id(timmy.id)
      .content(message_content.clone())
      .build();
    PrivateMessage::create(pool, &jess_timmy_message_form)
      .await
      .unwrap();

    let timmy_messages = PrivateMessageQuery {
      unread_only: false,
      creator_id: None,
      ..Default::default()
    }
    .list(pool, timmy.id)
    .await
    .unwrap();

    assert_length!(3, &timmy_messages);
    assert_eq!(timmy_messages[0].creator.id, jess.id);
    assert_eq!(timmy_messages[0].recipient.id, timmy.id);
    assert_eq!(timmy_messages[1].creator.id, timmy.id);
    assert_eq!(timmy_messages[1].recipient.id, sara.id);
    assert_eq!(timmy_messages[2].creator.id, sara.id);
    assert_eq!(timmy_messages[2].recipient.id, timmy.id);

    let timmy_unread_messages = PrivateMessageQuery {
      unread_only: true,
      creator_id: None,
      ..Default::default()
    }
    .list(pool, timmy.id)
    .await
    .unwrap();

    assert_length!(2, &timmy_unread_messages);
    assert_eq!(timmy_unread_messages[0].creator.id, jess.id);
    assert_eq!(timmy_unread_messages[0].recipient.id, timmy.id);
    assert_eq!(timmy_unread_messages[1].creator.id, sara.id);
    assert_eq!(timmy_unread_messages[1].recipient.id, timmy.id);

    let timmy_sara_messages = PrivateMessageQuery {
      unread_only: false,
      creator_id: Some(sara.id),
      ..Default::default()
    }
    .list(pool, timmy.id)
    .await
    .unwrap();

    assert_length!(2, &timmy_sara_messages);
    assert_eq!(timmy_sara_messages[0].creator.id, timmy.id);
    assert_eq!(timmy_sara_messages[0].recipient.id, sara.id);
    assert_eq!(timmy_sara_messages[1].creator.id, sara.id);
    assert_eq!(timmy_sara_messages[1].recipient.id, timmy.id);

    let timmy_sara_unread_messages = PrivateMessageQuery {
      unread_only: true,
      creator_id: Some(sara.id),
      ..Default::default()
    }
    .list(pool, timmy.id)
    .await
    .unwrap();

    assert_length!(1, &timmy_sara_unread_messages);
    assert_eq!(timmy_sara_unread_messages[0].creator.id, sara.id);
    assert_eq!(timmy_sara_unread_messages[0].recipient.id, timmy.id);

    // Make sure blocks are working
    let timmy_blocks_sara_form = PersonBlockForm {
      person_id: timmy.id,
      target_id: sara.id,
    };

    let inserted_block = PersonBlock::block(pool, &timmy_blocks_sara_form)
      .await
      .unwrap();

    let expected_block = PersonBlock {
      person_id: timmy.id,
      target_id: sara.id,
      published: inserted_block.published,
    };
    assert_eq!(expected_block, inserted_block);

    let timmy_messages = PrivateMessageQuery {
      unread_only: true,
      creator_id: None,
      ..Default::default()
    }
    .list(pool, timmy.id)
    .await
    .unwrap();

    assert_length!(1, &timmy_messages);

    let timmy_unread_messages = PrivateMessageView::get_unread_messages(pool, timmy.id)
      .await
      .unwrap();
    assert_eq!(timmy_unread_messages, 1);

    // This also deletes all persons and private messages thanks to sql `on delete cascade`
    Instance::delete(pool, instance.id).await.unwrap();
  }
}
