use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::{AdminBlockInstanceParams, AdminBlockInstanceResponse},
  utils::is_admin,
};
use lemmy_db_schema::source::federation_blocklist::{AdminBlockInstance, AdminBlockInstanceForm};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[tracing::instrument(skip(context))]
pub async fn block_instance(
  data: Json<AdminBlockInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<AdminBlockInstanceResponse>> {
  is_admin(&local_user_view)?;

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

  Ok(Json(AdminBlockInstanceResponse {
    blocked: data.block,
  }))
}
