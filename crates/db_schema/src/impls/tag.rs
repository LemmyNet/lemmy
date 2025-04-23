use crate::{
  newtypes::{CommunityId, TagId},
  source::tag::{Tag, TagInsertForm, TagUpdateForm},
  traits::Crud,
  utils::{get_conn, DbPool},
};
use chrono::Utc;
use diesel::{insert_into, ExpressionMethods, QueryDsl};
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

  pub async fn community_override_all_from_apub(
    pool: &mut DbPool<'_>,
    community_id: CommunityId,
    community_ap_id: String,
    ap_tags: Vec<TagInsertForm>,
  ) -> LemmyResult<()> {
    // Verify that each tag is actually in the given community.
    // Needed to ensure that incoming AP updates of one community can not manipulate tags in a
    // different community.
    let ap_tags: Vec<TagInsertForm> = ap_tags
      .into_iter()
      .filter(|tag| tag.ap_id.as_str().starts_with(&community_ap_id))
      .collect();

    let known_tags = Tag::get_by_community(pool, community_id).await?;
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
              updated: Some(Some(Utc::now())),
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
          updated: Some(Some(Utc::now())),
          ..Default::default()
        },
      )
      .await?;
    }

    Ok(())
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
