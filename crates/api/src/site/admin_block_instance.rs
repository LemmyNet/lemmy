use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::AdminBlockInstanceParams,
  utils::is_admin,
  LemmyErrorType,
  SuccessResponse,
};
use lemmy_db_schema::{
  source::{
    federation_blocklist::{FederationBlockList, FederationBlockListForm},
    instance::Instance,
    mod_log::admin::{AdminBlockInstance, AdminBlockInstanceForm},
  },
  traits::Crud,
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

  let instance_id = Instance::read_or_create(&mut context.pool(), data.instance.clone())
    .await?
    .id;
  let form = FederationBlockListForm {
    instance_id,
    expires: data.expires,
    updated: None,
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
