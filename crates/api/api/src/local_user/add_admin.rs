use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  source::{
    local_user::{LocalUser, LocalUserUpdateForm},
    mod_log::admin::{AdminAdd, AdminAddForm},
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::{
  api::{AddAdmin, AddAdminResponse},
  impls::PersonQuery,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn add_admin(
  data: Json<AddAdmin>,
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
    let admins = PersonQuery {
      admins_only: Some(true),
      ..Default::default()
    }
    .list(
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
  let form = AdminAddForm {
    mod_person_id: my_person_id,
    other_person_id: added_local_user.person.id,
    removed: Some(!data.added),
  };

  AdminAdd::create(&mut context.pool(), &form).await?;

  let admins = PersonQuery {
    admins_only: Some(true),
    ..Default::default()
  }
  .list(
    Some(my_person_id),
    local_user_view.person.instance_id,
    &mut context.pool(),
  )
  .await?;

  Ok(Json(AddAdminResponse { admins }))
}
