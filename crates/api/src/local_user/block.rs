use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{BlockPerson, BlockPersonResponse},
  utils::local_user_view_from_jwt,
};
use lemmy_db_schema::{
  source::person_block::{PersonBlock, PersonBlockForm},
  traits::Blockable,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[tracing::instrument(skip(context))]
pub async fn block_person(
  data: Json<BlockPerson>,
  context: Data<LemmyContext>,
) -> Result<Json<BlockPersonResponse>, LemmyError> {
  let local_user_view = local_user_view_from_jwt(&data.auth, &context).await?;

  let target_id = data.person_id;
  let person_id = local_user_view.person.id;

  // Don't let a person block themselves
  if target_id == person_id {
    return Err(LemmyErrorType::CantBlockYourself)?;
  }

  let person_block_form = PersonBlockForm {
    person_id,
    target_id,
  };

  let target_user = LocalUserView::read_person(&mut context.pool(), target_id).await;
  if target_user.map(|t| t.local_user.admin) == Ok(true) {
    return Err(LemmyErrorType::CantBlockAdmin)?;
  }

  if data.block {
    PersonBlock::block(&mut context.pool(), &person_block_form)
      .await
      .with_lemmy_type(LemmyErrorType::PersonBlockAlreadyExists)?;
  } else {
    PersonBlock::unblock(&mut context.pool(), &person_block_form)
      .await
      .with_lemmy_type(LemmyErrorType::PersonBlockAlreadyExists)?;
  }

  let person_view = PersonView::read(&mut context.pool(), target_id).await?;
  Ok(Json(BlockPersonResponse {
    person_view,
    blocked: data.block,
  }))
}
