use crate::{
  newtypes::CommunityPostTagId,
  schema::{community_post_tag, post_community_post_tag},
  source::community_post_tag::{
    CommunityPostTag,
    CommunityPostTagInsertForm,
    PostCommunityPostTagInsertForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use anyhow::Context;
use diesel::{insert_into, result::Error, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_utils::error::LemmyResult;

#[async_trait]
impl Crud for CommunityPostTag {
  type InsertForm = CommunityPostTagInsertForm;

  type UpdateForm = CommunityPostTagInsertForm;

  type IdType = CommunityPostTagId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_post_tag::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
  }

  async fn update(
    pool: &mut DbPool<'_>,
    pid: CommunityPostTagId,
    form: &Self::UpdateForm,
  ) -> Result<Self, Error> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(community_post_tag::table.find(pid))
      .set(form)
      .get_result::<Self>(conn)
      .await
  }
}

impl PostCommunityPostTagInsertForm {
  pub async fn insert_tag_associations(
    pool: &mut DbPool<'_>,
    tags: &[PostCommunityPostTagInsertForm],
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_community_post_tag::table)
      .values(tags)
      .execute(conn)
      .await
      .context("Failed to insert post community tag associations")?;
    Ok(())
  }
}
