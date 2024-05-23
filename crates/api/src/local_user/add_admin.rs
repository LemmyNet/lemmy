use actix_web::web::{Data, Json};
use lemmy_api_common::{
  context::LemmyContext,
  person::{AddAdmin, AddAdminResponse},
  utils::is_admin,
};
use lemmy_db_schema::{
  source::{
    local_user::{LocalUser, LocalUserUpdateForm},
    moderator::{ModAdd, ModAddForm},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult};

#[tracing::instrument(skip(context))]
pub async fn add_admin(
  data: Json<AddAdmin>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AddAdminResponse>> {
  // Make sure user is an admin
  is_admin(&local_user_view)?;

  // Make sure that the person_id added is local
  let added_local_user = LocalUserView::read_person(&mut context.pool(), data.person_id)
    .await?
    .ok_or(LemmyErrorType::ObjectNotLocal)?;

  LocalUser::update(
    &mut context.pool(),
    added_local_user.local_user.id,
    &LocalUserUpdateForm {
      admin: Some(data.added),
      ..Default::default()
    },
  )
  .await
  .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

  // Mod tables
  let form = ModAddForm {
    mod_person_id: local_user_view.person.id,
    other_person_id: added_local_user.person.id,
    removed: Some(!data.added),
  };

  ModAdd::create(&mut context.pool(), &form).await?;

  let admins = PersonView::admins(&mut context.pool()).await?;

  Ok(Json(AddAdminResponse { admins }))
}
