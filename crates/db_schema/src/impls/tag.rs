use crate::{
  newtypes::{CommunityId, TagId},
  source::tag::{PostTagInsertForm, Tag, TagInsertForm, TagUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, result::Error, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{post_tag, tag};
use lemmy_utils::error::LemmyResult;
impl Tag {
  pub async fn get_by_community(
    pool: &mut DbPool<'_>,
    search_community_id: CommunityId,
  ) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    tag::table
      .filter(tag::community_id.eq(search_community_id))
      .filter(tag::deleted.eq(false))
      .load::<Self>(conn)
      .await
  }
}

impl Crud for Tag {
  type InsertForm = TagInsertForm;

  type UpdateForm = TagUpdateForm;

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
