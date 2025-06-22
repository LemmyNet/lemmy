use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema::{
  source::instance::{InstanceActions, InstanceBlockForm},
  traits::Blockable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{SuccessResponse, UserBlockInstanceParams};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn user_block_instance(
  data: Json<UserBlockInstanceParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let instance_id = data.instance_id;
  let person_id = local_user_view.person.id;
  if local_user_view.person.instance_id == instance_id {
    return Err(LemmyErrorType::CantBlockLocalInstance)?;
  }

  let instance_block_form = InstanceBlockForm::new(person_id, instance_id);

  if data.block {
    InstanceActions::block(&mut context.pool(), &instance_block_form).await?;
  } else {
    InstanceActions::unblock(&mut context.pool(), &instance_block_form).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
