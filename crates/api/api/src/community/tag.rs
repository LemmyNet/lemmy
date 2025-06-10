use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{context::LemmyContext, utils::check_community_mod_action};
use lemmy_db_schema::{
  source::{
    community::Community,
    tag::{Tag, TagInsertForm, TagUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_community::api::{CreateCommunityTag, DeleteCommunityTag, UpdateCommunityTag};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::{error::LemmyResult, utils::validation::tag_name_length_check};

pub async fn create_community_tag(
  data: Json<CreateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  tag_name_length_check(&data.display_name)?;
  // Verify that only mods can create tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  // Create the tag
  let tag_form = TagInsertForm {
    display_name: data.display_name.clone(),
    community_id: data.community_id,
    ap_id: community.build_tag_ap_id(&data.display_name)?,
  };

  let tag = Tag::create(&mut context.pool(), &tag_form).await?;

  Ok(Json(tag))
}

pub async fn update_community_tag(
  data: Json<UpdateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can update tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  tag_name_length_check(&data.display_name)?;
  // Update the tag
  let tag_form = TagUpdateForm {
    display_name: Some(data.display_name.clone()),
    updated: Some(Some(Utc::now())),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  Ok(Json(tag))
}

pub async fn delete_community_tag(
  data: Json<DeleteCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can delete tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  // Soft delete the tag
  let tag_form = TagUpdateForm {
    updated: Some(Some(Utc::now())),
    deleted: Some(true),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  Ok(Json(tag))
}
