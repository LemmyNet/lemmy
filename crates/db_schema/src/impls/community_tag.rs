use crate::{
  diesel::SelectableHelper,
  newtypes::{CommunityId, CommunityTagId, PostId},
  source::{
    community_tag::{
      CommunityTag,
      CommunityTagInsertForm,
      CommunityTagUpdateForm,
      CommunityTagsView,
      PostCommunityTag,
      PostCommunityTagForm,
    },
    post::Post,
  },
};
use diesel::{
  ExpressionMethods,
  QueryDsl,
  delete,
  deserialize::FromSql,
  insert_into,
  pg::{Pg, PgValue},
  serialize::ToSql,
  sql_types::{Json, Nullable},
  upsert::excluded,
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_db_schema_file::schema::{community_tag, post_community_tag};
use lemmy_diesel_utils::{
  connection::{DbPool, get_conn},
  dburl::DbUrl,
  traits::Crud,
};
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::collections::HashSet;

impl Crud for CommunityTag {
  type InsertForm = CommunityTagInsertForm;
  type UpdateForm = CommunityTagUpdateForm;
  type IdType = CommunityTagId;

  async fn create(pool: &mut DbPool<'_>, form: &Self::InsertForm) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(community_tag::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  async fn update(
    pool: &mut DbPool<'_>,
    pid: CommunityTagId,
    form: &Self::UpdateForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    diesel::update(community_tag::table.find(pid))
      .set(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdate)
  }
}

impl CommunityTag {
  pub async fn read_for_community(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;
    community_tag::table
      .filter(community_tag::community_id.eq(community_id))
      .filter(community_tag::deleted.eq(false))
      .load::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn update_many(
    pool: &mut DbPool<'_>,
    mut forms: Vec<CommunityTagInsertForm>,
    existing_tags: Vec<CommunityTag>,
  ) -> LemmyResult<()> {
    let conn = &mut get_conn(pool).await?;
    let new_tag_ids = forms
      .iter()
      .map(|tag| tag.ap_id.clone())
      .collect::<HashSet<_>>();
    let delete_forms = existing_tags
      .into_iter()
      .filter(|tag| !new_tag_ids.contains(&tag.ap_id))
      .map(|t| CommunityTagInsertForm {
        ap_id: t.ap_id,
        name: t.name,
        display_name: None,
        community_id: t.community_id,
        deleted: Some(true),
        summary: None,
        color: Some(t.color),
      });
    forms.extend(delete_forms);

    conn
      .run_transaction(|conn| {
        async move {
          insert_into(community_tag::table)
            .values(&forms)
            .on_conflict(community_tag::ap_id)
            .do_update()
            .set((
              community_tag::display_name.eq(excluded(community_tag::display_name)),
              community_tag::summary.eq(excluded(community_tag::summary)),
              community_tag::deleted.eq(excluded(community_tag::deleted)),
            ))
            .execute(conn)
            .await?;

          Ok(())
        }
        .scope_boxed()
      })
      .await?;

    Ok(())
  }

  pub async fn read_for_post(
    pool: &mut DbPool<'_>,
    post_id: PostId,
  ) -> LemmyResult<Vec<CommunityTag>> {
    let conn = &mut get_conn(pool).await?;
    post_community_tag::table
      .inner_join(community_tag::table)
      .filter(post_community_tag::post_id.eq(post_id))
      .filter(community_tag::deleted.eq(false))
      .select(community_tag::all_columns)
      .get_results(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }

  pub async fn read_apub(pool: &mut DbPool<'_>, ap_id: &DbUrl) -> LemmyResult<CommunityTag> {
    let conn = &mut get_conn(pool).await?;
    community_tag::table
      .filter(community_tag::ap_id.eq(ap_id))
      .filter(community_tag::deleted.eq(false))
      .select(community_tag::all_columns)
      .get_result(conn)
      .await
      .with_lemmy_type(LemmyErrorType::NotFound)
  }
}

impl FromSql<Nullable<Json>, Pg> for CommunityTagsView {
  fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Json, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value::<CommunityTagsView>(value)?)
  }
  fn from_nullable_sql(
    bytes: Option<<Pg as diesel::backend::Backend>::RawValue<'_>>,
  ) -> diesel::deserialize::Result<Self> {
    match bytes {
      Some(bytes) => Self::from_sql(bytes),
      None => Ok(Self(vec![])),
    }
  }
}

impl ToSql<Nullable<Json>, Pg> for CommunityTagsView {
  fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<Json, Pg>>::to_sql(&value, &mut out.reborrow())
  }
}

impl PostCommunityTag {
  pub async fn update(
    pool: &mut DbPool<'_>,
    post: &Post,
    community_tag_ids: &[CommunityTagId],
  ) -> LemmyResult<Vec<Self>> {
    let conn = &mut get_conn(pool).await?;

    conn
      .run_transaction(|conn| {
        async move {
          delete(post_community_tag::table.filter(post_community_tag::post_id.eq(post.id)))
            .execute(conn)
            .await
            .with_lemmy_type(LemmyErrorType::Deleted)?;

          let forms = community_tag_ids
            .iter()
            .map(|tag_id| PostCommunityTagForm {
              post_id: post.id,
              community_tag_id: *tag_id,
            })
            .collect::<Vec<_>>();
          insert_into(post_community_tag::table)
            .values(forms)
            .returning(Self::as_select())
            .get_results(conn)
            .await
            .with_lemmy_type(LemmyErrorType::CouldntCreate)
        }
        .scope_boxed()
      })
      .await
  }
}
