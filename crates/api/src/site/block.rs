use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  context::LemmyContext,
  site::{BlockInstance, BlockInstanceResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::instance_block::{InstanceBlock, InstanceBlockForm},
  traits::Blockable,
};
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn block_instance(
  data: Json<BlockInstance>,
  context: Data<LemmyContext>,
) -> Result<Json<BlockInstanceResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let instance_id = data.instance_id;
  let person_id = local_user_view.person.id;
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
