use crate::{
  newtypes::{PostId, PostUrlId},
  source::post_url::{PostUrl, PostUrlInsertForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, QueryDsl, ExpressionMethods};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::post_url;

impl Crud for PostUrl {
  type InsertForm = PostUrlInsertForm;
  type UpdateForm = PostUrlInsertForm;
  type IdType = PostUrlId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_url::table)
      .values(form)
      .on_conflict((post_url::post_id, post_url::url))
      .do_update()
      .set(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    post_url_id: Self::IdType,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(post_url::table.find(post_url_id))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PostUrl {
  async fn list_from_post_id(post_id: PostId, pool: &mut DbPool<'_>) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    post_url::table
      .filter(post_url::post_id.eq(post_id))
      .order(post_url::page)
      .load::<Self>(conn)
      .await
  }
}
