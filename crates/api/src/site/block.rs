use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::{BlockInstance, BlockInstanceResponse},
};
use lemmy_db_schema::{
  source::instance_block::{InstanceBlock, InstanceBlockForm},
  traits::Blockable,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn block_instance(
  data: Json<BlockInstance>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> Result<Json<BlockInstanceResponse>, LemmyError> {
  let instance_id = data.instance_id;
  let person_id = local_user_view.person.id;
  if local_user_view.person.instance_id == instance_id {
    return Err(LemmyErrorType::CantBlockLocalInstance)?;
  }

  let instance_block_form = InstanceBlockForm {
    person_id,
    instance_id,
  };

  if data.block {
    InstanceBlock::block(&mut context.pool(), &instance_block_form)
      .await
      .with_lemmy_type(LemmyErrorType::InstanceBlockAlreadyExists)?;
  } else {
    InstanceBlock::unblock(&mut context.pool(), &instance_block_form)
      .await
      .with_lemmy_type(LemmyErrorType::InstanceBlockAlreadyExists)?;
  }

  Ok(Json(BlockInstanceResponse {
    blocked: data.block,
  }))
}
