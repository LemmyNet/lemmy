use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  source::person::{PersonActions, PersonBlockForm},
  traits::Blockable,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  PersonView,
  api::{BlockPerson, PersonResponse},
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn user_block_person(
  Json(data): Json<BlockPerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PersonResponse>> {
  check_local_user_valid(&local_user_view)?;
  let target_id = data.person_id;
  let my_person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;

  // Don't let a person block themselves
  if target_id == my_person_id {
    Err(LemmyErrorType::CantBlockYourself)?
  }

  let person_block_form = PersonBlockForm::new(my_person_id, target_id);

  let target_user = LocalUserView::read_person(&mut context.pool(), target_id)
    .await
    .ok();

  if target_user.is_some_and(|t| t.local_user.admin) {
    Err(LemmyErrorType::CantBlockAdmin)?
  }

  if data.block {
    PersonActions::block(&mut context.pool(), &person_block_form).await?;
  } else {
    PersonActions::unblock(&mut context.pool(), &person_block_form).await?;
  }

  let person_view = PersonView::read(
    &mut context.pool(),
    target_id,
    Some(my_person_id),
    local_instance_id,
    false,
  )
  .await?;
  Ok(Json(PersonResponse { person_view }))
}
