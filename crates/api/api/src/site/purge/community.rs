use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::is_admin,
};
use lemmy_db_schema::source::{
  community::Community,
  local_user::LocalUser,
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_schema_file::PersonId;
use lemmy_db_views_community::api::PurgeCommunity;
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::SuccessResponse;
use lemmy_diesel_utils::traits::Crud;
use lemmy_utils::error::LemmyResult;

pub async fn purge_community(
  Json(data): Json<PurgeCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<SuccessResponse>> {
  // Only let admin purge an item
  is_admin(&local_user_view)?;

  // Read the community to get its images
  let community = Community::read(&mut context.pool(), data.community_id).await?;

  // Also check that you're a higher admin than all the mods
  let community_mod_person_ids =
    CommunityModeratorView::for_community(&mut context.pool(), community.id)
      .await?
      .iter()
      .map(|cmv| cmv.moderator.id)
      .collect::<Vec<PersonId>>();

  LocalUser::is_higher_admin_check(
    &mut context.pool(),
    local_user_view.person.id,
    community_mod_person_ids,
  )
  .await?;

  Community::delete(&mut context.pool(), data.community_id).await?;

  // Mod tables
  let form = ModlogInsertForm::admin_purge_community(local_user_view.person.id, &data.reason);
  Modlog::create(&mut context.pool(), &[form]).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveCommunity {
      moderator: local_user_view.person.clone(),
      community,
      reason: data.reason.clone(),
      removed: true,
    },
    &context,
  )?;

  Ok(Json(SuccessResponse::default()))
}
