use crate::{
  newtypes::{PostId, TagId},
  schema::post_tag,
  source::{
    post_tag::{PostTag, PostTagForm},
    tag::PostTagInsertForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

impl PostTag {
  pub async fn delete_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> Result<(), diesel::result::Error> {
    let conn = &mut get_conn(pool).await?;
    delete(post_tag::table.filter(post_tag::post_id.eq(post_id)))
      .execute(conn)
      .await?;
    Ok(())
  }
}

impl Crud for PostTag {
  type InsertForm = PostTagInsertForm;
  type UpdateForm = PostTagForm;
  type IdType = (PostId, TagId);

  async fn create(
    pool: &mut DbPool<'_>,
    form: &PostTagInsertForm,
  ) -> Result<Self, diesel::result::Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_tag::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    _pool: &mut DbPool<'_>,
    _id: Self::IdType,
    _form: &Self::UpdateForm,
  ) -> Result<Self, diesel::result::Error> {
    Err(diesel::result::Error::QueryBuilderError(
      "PostTag does not support (create+delete only)".into(),
    ))
  }
}

impl PostTag {
  pub async fn create_many(
    pool: &mut DbPool<'_>,
    forms: Vec<PostTagInsertForm>,
  ) -> Result<(), diesel::result::Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_tag::table)
      .values(forms)
      .execute(conn)
      .await?;
    Ok(())
  }
}
