use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, RemoveCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    community::{Community, CommunityUpdateForm},
    moderator::{ModRemoveCommunity, ModRemoveCommunityForm},
  },
  traits::Crud,
};
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::time::naive_from_unix,
};

#[tracing::instrument(skip(context))]
pub async fn remove_community(
  data: Json<RemoveCommunity>,
  context: Data<LemmyContext>,
) -> Result<Json<CommunityResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  // Verify its an admin (only an admin can remove a community)
  is_admin(&local_user_view)?;

  // Do the remove
  let community_id = data.community_id;
  let removed = data.removed;
  let community = Community::update(
    &mut context.pool(),
    community_id,
    &CommunityUpdateForm::builder()
      .removed(Some(removed))
      .build(),
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunity)?;

  // Mod tables
  let expires = data.expires.map(naive_from_unix);
  let form = ModRemoveCommunityForm {
    mod_person_id: local_user_view.person.id,
    community_id: data.community_id,
    removed: Some(removed),
    reason: data.reason.clone(),
    expires,
  };
  ModRemoveCommunity::create(&mut context.pool(), &form).await?;

  ActivityChannel::submit_activity(
    SendActivityData::RemoveCommunity(
      local_user_view.person.clone(),
      community,
      data.reason.clone(),
      data.removed,
    ),
    &context,
  )
  .await?;

  build_community_response(&context, local_user_view, community_id).await
}
