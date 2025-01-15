use crate::{
  diesel::OptionalExtension,
  newtypes::{CommentId, PersonCommentMentionId, PersonId},
  schema::person_comment_mention,
  source::person_comment_mention::{
    PersonCommentMention,
    PersonCommentMentionInsertForm,
    PersonCommentMentionUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for PersonCommentMention {
  type InsertForm = PersonCommentMentionInsertForm;
  type UpdateForm = PersonCommentMentionUpdateForm;
  type IdType = PersonCommentMentionId;

  async fn create(
    pool: &mut DbPool<'_>,
    person_comment_mention_form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesn't return the existing row here
    insert_into(person_comment_mention::table)
      .values(person_comment_mention_form)
      .on_conflict((
        person_comment_mention::recipient_id,
        person_comment_mention::comment_id,
      ))
      .do_update()
      .set(person_comment_mention_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    person_comment_mention_id: PersonCommentMentionId,
    person_comment_mention_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person_comment_mention::table.find(person_comment_mention_id))
      .set(person_comment_mention_form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PersonCommentMention {
  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> Result<Vec<PersonCommentMention>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_comment_mention::table
        .filter(person_comment_mention::recipient_id.eq(for_recipient_id))
        .filter(person_comment_mention::read.eq(false)),
    )
    .set(person_comment_mention::read.eq(true))
    .get_results::<Self>(conn)
    .await
  }

  pub async fn read_by_comment_and_person(
    pool: &mut DbPool<'_>,
    for_comment_id: CommentId,
    for_recipient_id: PersonId,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    person_comment_mention::table
      .filter(person_comment_mention::comment_id.eq(for_comment_id))
      .filter(person_comment_mention::recipient_id.eq(for_recipient_id))
      .first(conn)
      .await
      .optional()
  }
}
