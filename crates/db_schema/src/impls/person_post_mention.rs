use crate::{
  diesel::OptionalExtension,
  newtypes::{PersonId, PersonPostMentionId, PostId},
  schema::person_post_mention,
  source::person_post_mention::{
    PersonPostMention,
    PersonPostMentionInsertForm,
    PersonPostMentionUpdateForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{dsl::insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

#[async_trait]
impl Crud for PersonPostMention {
  type InsertForm = PersonPostMentionInsertForm;
  type UpdateForm = PersonPostMentionUpdateForm;
  type IdType = PersonPostMentionId;

  async fn create(
    pool: &mut DbPool<'_>,
    person_post_mention_form: &Self::InsertForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    // since the return here isnt utilized, we dont need to do an update
    // but get_result doesn't return the existing row here
    insert_into(person_post_mention::table)
      .values(person_post_mention_form)
      .on_conflict((
        person_post_mention::recipient_id,
        person_post_mention::post_id,
      ))
      .do_update()
      .set(person_post_mention_form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    person_post_mention_id: PersonPostMentionId,
    person_post_mention_form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(person_post_mention::table.find(person_post_mention_id))
      .set(person_post_mention_form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PersonPostMention {
  pub async fn mark_all_as_read(
    pool: &mut DbPool<'_>,
    for_recipient_id: PersonId,
  ) -> Result<Vec<PersonPostMention>, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(
      person_post_mention::table
        .filter(person_post_mention::recipient_id.eq(for_recipient_id))
        .filter(person_post_mention::read.eq(false)),
    )
    .set(person_post_mention::read.eq(true))
    .get_results::<Self>(conn)
    .await
  }

  pub async fn read_by_post_and_person(
    pool: &mut DbPool<'_>,
    for_post_id: PostId,
    for_recipient_id: PersonId,
  ) -> Result<Option<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    person_post_mention::table
      .filter(person_post_mention::post_id.eq(for_post_id))
      .filter(person_post_mention::recipient_id.eq(for_recipient_id))
      .first(conn)
      .await
      .optional()
  }
}
