use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  community::{
    CommunityTagResponse,
    CreateCommunityTag,
    DeleteCommunityTag,
    ListCommunityTags,
    ListCommunityTagsResponse,
    UpdateCommunityTag,
  },
  context::LemmyContext,
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    tag::{Tag, TagInsertForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{error::LemmyResult, utils::validation::is_valid_tag_slug};
use url::Url;

#[tracing::instrument(skip(context))]
pub async fn create_community_tag(
  data: Json<CreateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityTagResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  // Verify that only mods can create tags
  check_community_mod_action(
    &local_user_view.person,
    &community,
    false,
    &mut context.pool(),
  )
  .await?;

  is_valid_tag_slug(&data.id_slug)?;

  // Create the tag
  let tag_form = TagInsertForm {
    name: data.name.clone(),
    community_id: data.community_id,
    ap_id: Url::parse(&format!("{}/tag/{}", community.actor_id, &data.id_slug))?.into(),
    published: None, // defaults to now
    updated: None,
    deleted: false,
  };

  let tag = Tag::create(&mut context.pool(), &tag_form).await?;

  Ok(Json(CommunityTagResponse {
    id: tag.id,
    name: tag.name,
    community_id: tag.community_id,
  }))
}

#[tracing::instrument(skip(context))]
pub async fn update_community_tag(
  data: Json<UpdateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityTagResponse>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can update tags
  check_community_mod_action(
    &local_user_view.person,
    &community,
    false,
    &mut context.pool(),
  )
  .await?;

  // Update the tag
  let tag_form = TagInsertForm {
    name: data.name.clone(),
    community_id: tag.community_id,
    ap_id: tag.ap_id,
    published: None,
    updated: Some(chrono::Utc::now()),
    deleted: false,
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  Ok(Json(CommunityTagResponse {
    id: tag.id,
    name: tag.name,
    community_id: tag.community_id,
  }))
}

#[tracing::instrument(skip(context))]
pub async fn delete_community_tag(
  data: Json<DeleteCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityTagResponse>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can delete tags
  check_community_mod_action(
    &local_user_view.person,
    &community,
    false,
    &mut context.pool(),
  )
  .await?;

  // Soft delete the tag
  let tag_form = TagInsertForm {
    name: tag.name.clone(),
    community_id: tag.community_id,
    ap_id: tag.ap_id,
    published: None,
    updated: Some(chrono::Utc::now()),
    deleted: true,
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  Ok(Json(CommunityTagResponse {
    id: tag.id,
    name: tag.name,
    community_id: tag.community_id,
  }))
}

#[tracing::instrument(skip(context))]
pub async fn list_community_tags(
  data: Json<ListCommunityTags>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<ListCommunityTagsResponse>> {
  let tags = Tag::get_by_community(&mut context.pool(), data.community_id).await?;

  let tag_responses = tags
    .into_iter()
    .map(|t| CommunityTagResponse {
      id: t.id,
      name: t.name,
      community_id: t.community_id,
    })
    .collect();

  Ok(Json(ListCommunityTagsResponse {
    tags: tag_responses,
  }))
}
