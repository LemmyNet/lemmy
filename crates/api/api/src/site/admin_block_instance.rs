use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_expire_time, is_admin},
};
use lemmy_db_schema::source::{
  federation_blocklist::{FederationBlockList, FederationBlockListForm},
  instance::Instance,
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{FederatedInstanceView, api::AdminBlockInstanceParams};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn admin_block_instance(
  Json(data): Json<AdminBlockInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<FederatedInstanceView>> {
  is_admin(&local_user_view)?;

  let expires_at = check_expire_time(data.expires_at)?;

  let allowlist = Instance::allowlist(&mut context.pool()).await?;
  if !allowlist.is_empty() {
    return Err(LemmyErrorType::CannotCombineFederationBlocklistAndAllowlist.into());
  }

  let instance_id = Instance::read_or_create(&mut context.pool(), &data.instance)
    .await?
    .id;

  let form = FederationBlockListForm::new(instance_id, expires_at);

  if data.block {
    FederationBlockList::block(&mut context.pool(), &form).await?;
  } else {
    FederationBlockList::unblock(&mut context.pool(), instance_id).await?;
  }

  let form = ModlogInsertForm::admin_block_instance(
    local_user_view.person.id,
    instance_id,
    data.block,
    &data.reason,
  );
  Modlog::create(&mut context.pool(), &[form]).await?;

  Ok(Json(
    FederatedInstanceView::read(&mut context.pool(), instance_id).await?,
  ))
}
