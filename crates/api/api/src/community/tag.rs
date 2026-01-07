use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, slur_regex},
};
use lemmy_db_schema::source::{
  community::Community,
  tag::{Tag, TagInsertForm, TagUpdateForm},
};
use lemmy_db_views_community::{
  CommunityView,
  api::{CreateCommunityTag, DeleteCommunityTag, UpdateCommunityTag},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{traits::Crud, utils::diesel_string_update};
use lemmy_utils::{
  error::LemmyResult,
  utils::{
    slurs::check_slurs,
    validation::{check_api_elements_count, description_length_check, is_valid_actor_name},
  },
};
use url::Url;

pub async fn create_community_tag(
  Json(data): Json<CreateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  is_valid_actor_name(&data.name)?;

  let community_view =
    CommunityView::read(&mut context.pool(), data.community_id, None, false).await?;
  let community = community_view.community;

  // Verify that only mods can create tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  check_api_elements_count(community_view.post_tags.0.len())?;
  if let Some(desc) = &data.description {
    description_length_check(desc)?;
    check_slurs(desc, &slur_regex(&context).await?)?;
  }

  let ap_id = Url::parse(&format!("{}/tag/{}", community.ap_id, &data.name))?;

  // Create the tag
  let tag_form = TagInsertForm {
    name: data.name.clone(),
    display_name: data.display_name.clone(),
    description: data.description.clone(),
    community_id: data.community_id,
    ap_id: ap_id.into(),
    deleted: Some(false),
  };

  let tag = Tag::create(&mut context.pool(), &tag_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

  Ok(Json(tag))
}

pub async fn update_community_tag(
  Json(data): Json<UpdateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can update tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  if let Some(desc) = &data.description {
    description_length_check(desc)?;
    check_slurs(desc, &slur_regex(&context).await?)?;
  }

  // Update the tag
  let tag_form = TagUpdateForm {
    display_name: diesel_string_update(data.display_name.as_deref()),
    description: diesel_string_update(data.description.as_deref()),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;
  Ok(Json(tag))
}

pub async fn delete_community_tag(
  Json(data): Json<DeleteCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can delete tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  // Soft delete the tag
  let tag_form = TagUpdateForm {
    updated_at: Some(Some(Utc::now())),
    deleted: Some(true),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

  Ok(Json(tag))
}
