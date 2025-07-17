use crate::{
  newtypes::{CommunityId, PostId, TagId},
  source::{
    community::Community,
    tag::{Tag, TagInsertForm, TagUpdateForm, TagsView},
  },
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{
  deserialize::FromSql,
  insert_into,
  pg::{Pg, PgValue},
  serialize::ToSql,
  sql_types::{Json, Nullable},
  ExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema_file::schema::tag;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};
use std::collections::{HashMap, HashSet};

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

  pub async fn community_update_from_apub(
    pool: &mut DbPool<'_>,
    community: &Community,
    ap_tags: Vec<TagInsertForm>,
  ) -> LemmyResult<()> {
    // Verify that each tag is actually in the given community.
    // Needed to ensure that incoming AP updates of one community can not manipulate tags in a
    // different community.
    let ap_tags: Vec<TagInsertForm> = ap_tags
      .into_iter()
      .filter(|tag| tag.ap_id.as_str().starts_with(community.ap_id.as_ref()))
      .collect();

    let known_tags = Tag::get_by_community(pool, community.id).await?;
    let old_tags = known_tags
      .iter()
      .map(|tag| (tag.ap_id.clone(), tag))
      .collect::<HashMap<_, _>>();
    let new_tag_ids = ap_tags
      .iter()
      .map(|tag| tag.ap_id.clone())
      .collect::<HashSet<_>>();

    let to_delete = known_tags
      .iter()
      .filter(|tag| !new_tag_ids.contains(&tag.ap_id))
      .map(|tag| tag.id)
      .collect::<Vec<_>>();
    let to_insert = ap_tags
      .iter()
      .filter(|tag| !old_tags.contains_key(&tag.ap_id))
      .collect::<Vec<_>>();
    for tag in to_insert {
      Tag::create(pool, tag).await?;
    }
    // if display name is different, we need to update it
    for tag in ap_tags {
      if let Some(old_tag) = old_tags.get(&tag.ap_id) {
        if old_tag.display_name != tag.display_name {
          Tag::update(
            pool,
            old_tag.id,
            &TagUpdateForm {
              display_name: Some(tag.display_name.clone()),
              updated_at: Some(Some(Utc::now())),
              ..Default::default()
            },
          )
          .await?;
        }
      }
    }
    for tag in to_delete {
      Tag::update(
        pool,
        tag,
        &TagUpdateForm {
          deleted: Some(true),
          updated_at: Some(Some(Utc::now())),
          ..Default::default()
        },
      )
      .await?;
    }

    Ok(())
  }

  pub async fn read_for_post(pool: &mut DbPool<'_>, post_id: PostId) -> LemmyResult<Vec<Self>> {
    todo!()
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

impl FromSql<Nullable<Json>, Pg> for TagsView {
  fn from_sql(bytes: PgValue) -> diesel::deserialize::Result<Self> {
    let value = <serde_json::Value as FromSql<Json, Pg>>::from_sql(bytes)?;
    Ok(serde_json::from_value::<TagsView>(value)?)
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

impl ToSql<Nullable<Json>, Pg> for TagsView {
  fn to_sql(&self, out: &mut diesel::serialize::Output<Pg>) -> diesel::serialize::Result {
    let value = serde_json::to_value(self)?;
    <serde_json::Value as ToSql<Json, Pg>>::to_sql(&value, &mut out.reborrow())
  }
}
