use crate::{
  newtypes::{CommunityId, TagId},
  source::tag::{Tag, TagInsertForm, TagUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::tag;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

impl Tag {
  pub async fn get_by_community(
    pool: &mut DbPool<'_>,
    search_community_id: CommunityId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    tag::table
      .filter(tag::community_id.eq(search_community_id))
      .filter(tag::deleted.eq(false))
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl Crud for Tag {
  type InsertForm = TagInsertForm;
  type UpdateForm = TagUpdateForm;
  type IdType = TagId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(tag::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreateTag)
  }

  async fn update(pool: &mut DbPool<'_>, pid: TagId, form: &Self::UpdateForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(tag::table.find(pid))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateTag)
  }
}
