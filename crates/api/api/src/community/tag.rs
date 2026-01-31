use activitypub_federation::config::Data;
use actix_web::web::{Json, Query};
use chrono::Utc;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_mod_or_admin, is_mod_or_admin_opt, slur_regex},
};
use lemmy_db_schema::source::{
  community::Community,
  community_tag::{CommunityTag, CommunityTagInsertForm, CommunityTagUpdateForm},
};
use lemmy_db_views_community::{
  CommunityView,
  api::{CreateCommunityTag, DeleteCommunityTag, EditCommunityTag, ListCommunityTags},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{traits::Crud, utils::diesel_string_update};
use lemmy_utils::{
  error::LemmyResult,
  utils::{
    slurs::check_slurs,
    validation::{check_max_tags_count, is_valid_actor_name, summary_length_check},
  },
};
use url::Url;

pub async fn list_community_tags(
  Query(data): Query<ListCommunityTags>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<Vec<CommunityTag>>> {
  let community_id = data.community_id;
  let is_mod_or_admin = is_mod_or_admin_opt(
    &mut context.pool(),
    local_user_view.as_ref(),
    Some(community_id),
  )
  .await
  .is_ok();

  let tags = CommunityTag::list(&mut context.pool(), community_id, is_mod_or_admin).await?;

  Ok(Json(tags))
}

pub async fn create_community_tag(
  Json(data): Json<CreateCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityTag>> {
  let community_id = data.community_id;

  is_valid_actor_name(&data.name)?;

  let community_view = CommunityView::read(&mut context.pool(), community_id, None, false).await?;
  let community = community_view.community;

  // Verify that only mods or admins can create tags
  is_mod_or_admin(&mut context.pool(), &local_user_view, community_id).await?;

  // Check to make sure there aren't too many tags
  let tags_count = CommunityTag::count(&mut context.pool(), community_id).await?;
  check_max_tags_count(tags_count)?;

  if let Some(summary) = &data.summary {
    summary_length_check(summary)?;
    check_slurs(summary, &slur_regex(&context).await?)?;
  }

  let ap_id = Url::parse(&format!("{}/tag/{}", community.ap_id, &data.name))?;

  // Create the tag
  let tag_form = CommunityTagInsertForm {
    name: data.name.clone(),
    display_name: data.display_name.clone(),
    summary: data.summary.clone(),
    community_id: data.community_id,
    ap_id: ap_id.into(),
    deleted: Some(false),
    color: data.color,
  };

  let tag = CommunityTag::create(&mut context.pool(), &tag_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

  Ok(Json(tag))
}

pub async fn edit_community_tag(
  Json(data): Json<EditCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityTag>> {
  let tag = CommunityTag::read(&mut context.pool(), data.tag_id).await?;

  // Verify that only mods can update tags
  is_mod_or_admin(&mut context.pool(), &local_user_view, tag.community_id).await?;

  if let Some(summary) = &data.summary {
    summary_length_check(summary)?;
    check_slurs(summary, &slur_regex(&context).await?)?;
  }

  // Update the tag
  let tag_form = CommunityTagUpdateForm {
    display_name: diesel_string_update(data.display_name.as_deref()),
    summary: diesel_string_update(data.summary.as_deref()),
    updated_at: Some(Some(Utc::now())),
    color: data.color,
    ..Default::default()
  };

  let tag = CommunityTag::update(&mut context.pool(), data.tag_id, &tag_form).await?;
  Ok(Json(tag))
}

pub async fn delete_community_tag(
  Json(data): Json<DeleteCommunityTag>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityTag>> {
  let tag = CommunityTag::read(&mut context.pool(), data.tag_id).await?;
  let community = Community::read(&mut context.pool(), tag.community_id).await?;

  // Verify that only mods can delete tags
  is_mod_or_admin(&mut context.pool(), &local_user_view, tag.community_id).await?;

  // Soft delete the tag
  let tag_form = CommunityTagUpdateForm {
    updated_at: Some(Some(Utc::now())),
    deleted: Some(data.delete),
    ..Default::default()
  };

  let tag = CommunityTag::update(&mut context.pool(), data.tag_id, &tag_form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

  Ok(Json(tag))
}
