use crate::{
  diesel::SelectableHelper,
  newtypes::{CommunityId, PostId, TagId},
  source::{
    post_tag::{PostTag, PostTagForm},
    tag::PostTagInsertForm,
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use diesel::{delete, insert_into, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::{post_tag, tag};
use url::Url;

impl PostTag {
  pub async fn set(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    tags: Vec<PostTagInsertForm>,
  ) -> Result<Vec<Self>, diesel::result::Error> {
    PostTag::delete_for_post(pool, post_id).await?;
    PostTag::create_many(pool, tags).await
  }
  pub async fn set_from_apub(
    pool: &mut DbPool<'_>,
    post_id: PostId,
    tag_ap_ids: Vec<Url>,
    community_id: CommunityId,
  ) -> Result<Vec<Self>, diesel::result::Error> {
    // find tags in table. this also filters out tags we don't know about or that don't belong to
    // the right community
    let looked_up_ids = {
      let conn = &mut get_conn(pool).await?;
      tag::table
        .filter(tag::community_id.eq(community_id))
        .filter(tag::ap_id.eq_any(tag_ap_ids.iter().map(|ap_id| ap_id.as_str())))
        .select(tag::id)
        .get_results::<TagId>(conn)
        .await?
    };
    let tags = looked_up_ids
      .into_iter()
      .map(|tag_id| PostTagInsertForm { post_id, tag_id })
      .collect();
    PostTag::set(pool, post_id, tags).await
  }
  async fn delete_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> Result<usize, diesel::result::Error> {
    let conn = &mut get_conn(pool).await?;
    delete(post_tag::table.filter(post_tag::post_id.eq(post_id)))
      .execute(conn)
      .await
  }
  pub async fn create_many(
    pool: &mut DbPool<'_>,
    forms: Vec<PostTagInsertForm>,
  ) -> Result<Vec<Self>, diesel::result::Error> {
    let conn = &mut get_conn(pool).await?;
    insert_into(post_tag::table)
      .values(forms)
      .returning(Self::as_select())
      .get_results(conn)
      .await
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
