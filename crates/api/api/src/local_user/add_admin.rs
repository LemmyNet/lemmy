use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, notify::notify_mod_action, utils::is_admin};
use lemmy_db_schema::source::{
  local_user::{LocalUser, LocalUserUpdateForm},
  modlog::{Modlog, ModlogInsertForm},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  PersonView,
  api::{AddAdmin, AddAdminResponse},
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn add_admin(
  Json(data): Json<AddAdmin>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddAdminResponse>> {
  let my_person_id = local_user_view.person.id;

  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // If its an admin removal, also check that you're a higher admin
  if !data.added {
    LocalUser::is_higher_admin_check(&mut context.pool(), my_person_id, vec![data.person_id])
      .await?;

    // Dont allow removing the last admin
    let admins = PersonView::list_admins(
      None,
      local_user_view.person.instance_id,
      &mut context.pool(),
    )
    .await?;
    if admins.len() == 1 {
      Err(LemmyErrorType::CannotLeaveAdmin)?
    }
  }

  // Make sure that the person_id added is local
  let added_local_user = LocalUserView::read_person(&mut context.pool(), data.person_id).await?;

  LocalUser::update(
    &mut context.pool(),
    added_local_user.local_user.id,
    &LocalUserUpdateForm {
      admin: Some(data.added),
      ..Default::default()
    },
  )
  .await?;

  // Mod tables
  let form = ModlogInsertForm::admin_add(
    &local_user_view.person,
    added_local_user.person.id,
    !data.added,
  );
  let action = Modlog::create(&mut context.pool(), &[form]).await?;
  notify_mod_action(action.clone(), &context);

  let admins = PersonView::list_admins(
    Some(my_person_id),
    local_user_view.person.instance_id,
    &mut context.pool(),
  )
  .await?;

  Ok(Json(AddAdminResponse { admins }))
}
