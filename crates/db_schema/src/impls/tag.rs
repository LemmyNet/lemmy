use crate::{
  newtypes::TagId,
  schema::{post_tag, tag},
  source::tag::{PostTagInsertForm, Tag, TagInsertForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::LemmyResult;

#[async_trait]
impl Crud for Tag {
  type InsertForm = TagInsertForm;

  type UpdateForm = TagInsertForm;

  type IdType = TagId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(tag::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    pid: TagId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(tag::table.find(pid))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PostTagInsertForm {
  pub async fn insert_tag_associations(
    pool: &mut DbPool<'_>,
    tags: &[PostTagInsertForm],
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_tag::table)
      .values(tags)
      .execute(conn)
      .await?;
    Ok(())
  }
}
