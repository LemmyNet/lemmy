use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  build_response::build_community_response,
  context::LemmyContext,
  notify::notify_mod_action,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, is_admin},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    community_report::CommunityReport,
    mod_log::admin::{AdminRemoveCommunity, AdminRemoveCommunityForm},
  },
  traits::{Crud, Reportable},
};
use lemmy_db_views_community::api::{CommunityResponse, RemoveCommunity};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn remove_community(
  data: Json<RemoveCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  let community = Community::read(&mut context.pool(), data.community_id).await?;
  check_community_mod_action(&local_user_view, &community, true, &mut context.pool()).await?;

  // Verify its an admin (only an admin can remove a community)
  is_admin(&local_user_view)?;

  // Do the remove
  let community_id = data.community_id;
  let removed = data.removed;
  let community = Community::update(
    &mut context.pool(),
    community_id,
    &CommunityUpdateForm {
      removed: Some(removed),
      ..Default::default()
    },
  )
  .await?;

  CommunityReport::resolve_all_for_object(
    &mut context.pool(),
    community_id,
    local_user_view.person.id,
  )
  .await?;

  // Mod tables
  let form = AdminRemoveCommunityForm {
    mod_person_id: local_user_view.person.id,
    community_id: data.community_id,
    removed: Some(removed),
    reason: data.reason.clone(),
  };
  let action = AdminRemoveCommunity::create(&mut context.pool(), &form).await?;
  for m in CommunityModeratorView::for_community(&mut context.pool(), data.community_id).await? {
    notify_mod_action(action.clone(), m.moderator.id, context.app_data());
  }

  ActivityChannel::submit_activity(
    SendActivityData::RemoveCommunity {
      moderator: local_user_view.person.clone(),
      community,
      reason: data.reason.clone(),
      removed: data.removed,
    },
    &context,
  )?;

  build_community_response(&context, local_user_view, community_id).await
}
