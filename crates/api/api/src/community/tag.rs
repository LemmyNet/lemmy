use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::check_community_mod_action,
};
use lemmy_db_schema::{
  source::{
    community::Community,
    tag::{Tag, TagInsertForm, TagUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_community::api::{CreateCommunityTag, DeleteCommunityTag, UpdateCommunityTag};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{error::LemmyResult, utils::validation::is_valid_actor_name};
use url::Url;

pub async fn create_community_tag(
  data: Json<CreateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  // reuse this existing function for validation
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;
  is_valid_actor_name(&data.name, local_site.actor_name_max_length)?;

  let community = Community::read(&mut context.pool(), data.community_id).await?;

  // Verify that only mods can create tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  let ap_id = Url::parse(&format!("{}/tag/{}", community.ap_id, &data.name))?;

  // Create the tag
  let tag_form = TagInsertForm {
    name: data.name.clone(),
    display_name: data.display_name.clone(),
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
  data: Json<UpdateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Tag>> {
  let tag = Tag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can update tags
  check_community_mod_action(&local_user_view, &community, false, &mut context.pool()).await?;

  // Update the tag
  let tag_form = TagUpdateForm {
    display_name: data.display_name.clone(),
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let tag = Tag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

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
