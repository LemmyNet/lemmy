use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{AddAdmin, AddAdminResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModAdd, ModAddForm}, local_user::{LocalUserUpdateForm, LocalUser},
  },
  traits::Crud,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for AddAdmin {
  type Response = AddAdminResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let target = LocalUserView::read_person(context.pool(), data.person_id).await?;
    LocalUser::update(
      context.pool(),
      target.local_user.id,
      &LocalUserUpdateForm::builder().admin(Some(data.added)).build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_user"))?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: target.person.id,
      removed: Some(!data.added),
    };

    ModAdd::create(context.pool(), &form).await?;

    let admins = PersonView::admins(context.pool()).await?;

    Ok(AddAdminResponse { admins })
  }
}
