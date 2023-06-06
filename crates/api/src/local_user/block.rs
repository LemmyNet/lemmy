use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BlockPerson, BlockPersonResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::person_block::{PersonBlock, PersonBlockForm},
  traits::Blockable,
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for BlockPerson {
  type Response = BlockPersonResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<BlockPersonResponse, LemmyError> {
    let data: &BlockPerson = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let target_id = data.person_id;
    let person_id = local_user_view.person.id;

    // Don't let a person block themselves
    if target_id == person_id {
      return Err(LemmyError::from_message("cant_block_yourself"));
    }

    let person_block_form = PersonBlockForm {
      person_id,
      target_id,
    };

    let target_person_view = PersonView::read(context.pool(), target_id).await?;

    if target_person_view.person.admin {
      return Err(LemmyError::from_message("cant_block_admin"));
    }

    if data.block {
      PersonBlock::block(context.pool(), &person_block_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "person_block_already_exists"))?;
    } else {
      PersonBlock::unblock(context.pool(), &person_block_form)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "person_block_already_exists"))?;
    }

    Ok(BlockPersonResponse {
      person_view: target_person_view,
      blocked: data.block,
    })
  }
}
