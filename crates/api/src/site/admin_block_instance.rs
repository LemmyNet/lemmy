use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::AdminBlockInstanceParams,
  utils::is_admin,
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::source::{
  federation_blocklist::{AdminBlockInstance, AdminBlockInstanceForm},
  instance::Instance,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn admin_block_instance(
  data: Json<AdminBlockInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  let allowlist = Instance::allowlist(&mut context.pool()).await?;
  if !allowlist.is_empty() {
    Err(LemmyErrorType::CannotCombineFederationBlocklistAndAllowlist)?;
  }

  let instance_block_form = AdminBlockInstanceForm {
    instance_id: data.instance_id,
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    expires: data.expires,
  };

  if data.block {
    AdminBlockInstance::block(&mut context.pool(), &instance_block_form).await?;
  } else {
    AdminBlockInstance::unblock(&mut context.pool(), data.instance_id).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
