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
  federation_allowlist::{FederationAllowList, FederationAllowListForm},
  instance::Instance,
  mod_log::admin::{AdminAllowInstance, AdminAllowInstanceForm},
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

  let instance_id = Instance::read_or_create(&mut context.pool(), data.instance.clone())
    .await?
    .id;
  let form = FederationAllowListForm {
    instance_id,
    updated: None,
  };
  if data.allow {
    FederationAllowList::allow(&mut context.pool(), &form).await?;
  } else {
    FederationAllowList::unallow(&mut context.pool(), instance_id).await?;
  }

  let mod_log_form = AdminAllowInstanceForm {
    instance_id,
    admin_person_id: local_user_view.person.id,
    reason: data.reason.clone(),
    allowed: data.allow,
  };
  AdminAllowInstance::insert(&mut context.pool(), &mod_log_form).await?;

  Ok(Json(SuccessResponse::default()))
}
