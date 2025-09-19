use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::source::instance::{
  InstanceActions,
  InstanceCommunitiesBlockForm,
  InstancePersonsBlockForm,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::api::{
  SuccessResponse,
  UserBlockInstanceCommunitiesParams,
  UserBlockInstancePersonsParams,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn user_block_instance_communities(
  data: Json<UserBlockInstanceCommunitiesParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  check_local_user_valid(&local_user_view)?;
  let instance_id = data.instance_id;
  let person_id = local_user_view.person.id;
  if local_user_view.person.instance_id == instance_id {
    return Err(LemmyErrorType::CantBlockLocalInstance)?;
  }

  let block_form = InstanceCommunitiesBlockForm::new(person_id, instance_id);

  if data.block {
    InstanceActions::block_communities(&mut context.pool(), &block_form).await?;
  } else {
    InstanceActions::unblock_communities(&mut context.pool(), &block_form).await?;
  }

  Ok(Json(SuccessResponse::default()))
}

pub async fn user_block_instance_persons(
  data: Json<UserBlockInstancePersonsParams>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<SuccessResponse>> {
  let instance_id = data.instance_id;
  let person_id = local_user_view.person.id;
  if local_user_view.person.instance_id == instance_id {
    return Err(LemmyErrorType::CantBlockLocalInstance)?;
  }

  let block_form = InstancePersonsBlockForm::new(person_id, instance_id);

  if data.block {
    InstanceActions::block_persons(&mut context.pool(), &block_form).await?;
  } else {
    InstanceActions::unblock_persons(&mut context.pool(), &block_form).await?;
  }

  Ok(Json(SuccessResponse::default()))
}
