use crate::{
  diesel::OptionalExtension,
  newtypes::{CommentId, PersonId, PersonMentionId},
  schema::person_mention,
  source::person_mention::{PersonMention, PersonMentionInsertForm, PersonMentionUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for PersonMention {
  type InsertForm = PersonMentionInsertForm;
  type UpdateForm = PersonMentionUpdateForm;
  type IdType = PersonMentionId;

  async fn create(
    pool: &mut DbPool<'_>,
    person_mention_form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesn't return the existing row here
    insert_into(person_mention::table)
      .values(person_mention_form)
      .on_conflict((person_mention::recipient_id, person_mention::comment_id))
      .do_update()
      .set(person_mention_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    person_mention_id: PersonMentionId,
    person_mention_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person_mention::table.find(person_mention_id))
      .set(person_mention_form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PersonMention {
  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> Result<Vec<PersonMention>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_mention::table
        .filter(person_mention::recipient_id.eq(for_recipient_id))
        .filter(person_mention::read.eq(false)),
    )
    .set(person_mention::read.eq(true))
    .get_results::<Self>(conn)
    .await
  }

  pub async fn read_by_comment_and_person(
    pool: &mut DbPool<'_>,
    for_comment_id: CommentId,
    for_recipient_id: PersonId,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    person_mention::table
      .filter(person_mention::comment_id.eq(for_comment_id))
      .filter(person_mention::recipient_id.eq(for_recipient_id))
      .first(conn)
      .await
      .optional()
  }
}
