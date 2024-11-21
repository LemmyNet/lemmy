use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::AdminAllowInstanceParams,
  utils::is_admin,
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::source::{
  federation_allowlist::{AdminAllowInstance, AdminAllowInstanceForm},
  instance::Instance,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn admin_allow_instance(
  data: Json<AdminAllowInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  is_admin(&local_user_view)?;

  let blocklist = Instance::blocklist(&mut context.pool()).await?;
  if !blocklist.is_empty() {
    Err(LemmyErrorType::CannotCombineFederationBlocklistAndAllowlist)?;
  }

  let instance_block_form = AdminAllowInstanceForm {
    instance_id: data.instance_id,
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
  };

  if data.allow {
    AdminAllowInstance::allow(&mut context.pool(), &instance_block_form).await?;
  } else {
    AdminAllowInstance::unallow(&mut context.pool(), data.instance_id).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
