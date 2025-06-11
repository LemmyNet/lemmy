use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  source::{
    federation_allowlist::{FederationAllowList, FederationAllowListForm},
    instance::Instance,
    mod_log::admin::{AdminAllowInstance, AdminAllowInstanceForm},
  },
  traits::Crud,
};
use lemmy_db_views_api_misc::SuccessResponse;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::AdminAllowInstanceParams;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

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
    updated_at: None,
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
  AdminAllowInstance::create(&mut context.pool(), &mod_log_form).await?;

  Ok(Json(SuccessResponse::default()))
}
