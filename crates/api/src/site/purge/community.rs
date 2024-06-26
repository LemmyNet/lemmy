use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  request::purge_image_from_pictrs,
  send_activity::{ActivityChannel, SendActivityData},
  site::PurgeCommunity,
  utils::{check_is_higher_admin, is_admin, purge_image_posts_for_community},
  SuccessResponse,
};
use lemmy_db_schema::{
  newtypes::PersonId,
  source::{
    community::Community,
    moderator::{AdminPurgeCommunity, AdminPurgeCommunityForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{error::LemmyResult, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn purge_community(
  data: Json<PurgeCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the community to get its images
  let community = Community::read(&mut context.pool(), data.community_id)
    .await?
    .ok_or(LemmyErrorType::CouldntFindCommunity)?;

  // Also check that you're a higher admin than all the mods
  let community_mod_person_ids =
    CommunityModeratorView::for_community(&mut context.pool(), community.id)
      .await?
      .iter()
      .map(|cmv| cmv.moderator.id)
      .collect::<Vec<PersonId>>();

  check_is_higher_admin(
    &mut context.pool(),
    &local_user_view,
    &community_mod_person_ids,
  )
  .await?;

  if let Some(banner) = &community.banner {
    purge_image_from_pictrs(banner, &context).await.ok();
  }

  if let Some(icon) = &community.icon {
    purge_image_from_pictrs(icon, &context).await.ok();
  }

  purge_image_posts_for_community(data.community_id, &context).await?;

  Community::delete(&mut context.pool(), data.community_id).await?;

  // Mod tables
  let form = AdminPurgeCommunityForm {
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
  };
  AdminPurgeCommunity::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveCommunity {
      moderator: local_user_view.person.clone(),
      community,
      reason: data.reason.clone(),
      removed: true,
    },
    &context,
  )
  .await?;

  Ok(Json(SuccessResponse::default()))
}
