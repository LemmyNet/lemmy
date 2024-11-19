use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::{AdminBlockInstance, AdminBlockInstanceResponse},
  utils::is_admin,
};
use lemmy_db_schema::source::federation_blocklist::{FederationBlockList, FederationBlockListForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn block_instance(
  data: Json<AdminBlockInstance>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<AdminBlockInstanceResponse>> {
  is_admin(&local_user_view)?;

  let instance_block_form = FederationBlockListForm {
    instance_id: data.instance_id,
    admin_person_id: Some(local_user_view.person.id),
    reason: data.reason.clone(),
    expires: data.expires,
    updated: None,
  };

  if data.block {
    FederationBlockList::block(&mut context.pool(), &instance_block_form).await?;
  } else {
    FederationBlockList::unblock(&mut context.pool(), &instance_block_form).await?;
  }

  Ok(Json(AdminBlockInstanceResponse {
    blocked: data.block,
  }))
}
