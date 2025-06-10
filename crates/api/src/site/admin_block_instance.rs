use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  source::{
    federation_blocklist::{FederationBlockList, FederationBlockListForm},
    instance::Instance,
    mod_log::admin::{AdminBlockInstance, AdminBlockInstanceForm},
  },
  traits::Crud,
};
use lemmy_db_views_api_misc::SuccessResponse;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::AdminBlockInstanceParams;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

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

  let instance_id = Instance::read_or_create(&mut context.pool(), data.instance.clone())
    .await?
    .id;
  let form = FederationBlockListForm {
    instance_id,
    expires_at: data.expires_at,
    updated_at: None,
  };

  if data.block {
    FederationBlockList::block(&mut context.pool(), &form).await?;
  } else {
    FederationBlockList::unblock(&mut context.pool(), instance_id).await?;
  }

  let mod_log_form = AdminBlockInstanceForm {
    instance_id,
    admin_person_id: local_user_view.person.id,
    blocked: data.block,
    reason: data.reason.clone(),
  };
  AdminBlockInstance::create(&mut context.pool(), &mod_log_form).await?;

  Ok(Json(SuccessResponse::default()))
}
