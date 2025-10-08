use actix_web::web::{Data, Json};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_local_user_valid, get_url_blocklist, process_markdown, slur_regex},
};
use lemmy_db_schema::source::person::{PersonActions, PersonNoteForm};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  api::{NotePerson, PersonResponse},
  PersonView,
};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::{slurs::check_slurs, validation::is_valid_body_field},
};

pub async fn user_note_person(
  data: Json<NotePerson>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<PersonResponse>> {
  check_local_user_valid(&local_user_view)?;
  let target_id = data.person_id;
  let my_person_id = local_user_view.person.id;
  let local_instance_id = local_user_view.person.instance_id;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;

  // Don't let a person note themselves
  if target_id == my_person_id {
    Err(LemmyErrorType::CantNoteYourself)?
  }

  // If the note is empty, delete it
  if data.note.is_empty() {
    PersonActions::delete_note(&mut context.pool(), my_person_id, target_id).await?;
  } else {
    check_slurs(&data.note, &slur_regex)?;
    is_valid_body_field(&data.note, false)?;

    let note = process_markdown(&data.note, &slur_regex, &url_blocklist, &context).await?;
    let note_form = PersonNoteForm::new(my_person_id, target_id, note);

    PersonActions::note(&mut context.pool(), &note_form).await?;
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
