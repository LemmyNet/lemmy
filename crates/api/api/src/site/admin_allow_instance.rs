use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::{
  federation_allowlist::{FederationAllowList, FederationAllowListForm},
  instance::Instance,
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::{FederatedInstanceView, api::AdminAllowInstanceParams};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn admin_allow_instance(
  data: Json<AdminAllowInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<FederatedInstanceView>> {
  is_admin(&local_user_view)?;

  let blocklist = Instance::blocklist(&mut context.pool()).await?;
  if !blocklist.is_empty() {
    Err(LemmyErrorType::CannotCombineFederationBlocklistAndAllowlist)?;
  }

  let instance_id = Instance::read_or_create(&mut context.pool(), &data.instance)
    .await?
    .id;
  let form = FederationAllowListForm::new(instance_id);
  if data.allow {
    FederationAllowList::allow(&mut context.pool(), &form).await?;
  } else {
    FederationAllowList::unallow(&mut context.pool(), instance_id).await?;
  }

  let form = ModlogInsertForm::admin_allow_instance(
    local_user_view.person.id,
    instance_id,
    data.allow,
    &data.reason,
  );
  Modlog::create(&mut context.pool(), &[form]).await?;

  Ok(Json(
    FederatedInstanceView::read(&mut context.pool(), instance_id).await?,
  ))
}
