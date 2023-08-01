use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{AddAdmin, AddAdminResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    local_user::{LocalUser, LocalUserUpdateForm},
    moderator::{ModAdd, ModAddForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::{LemmyError, LemmyErrorExt, LemmyErrorType};

#[async_trait::async_trait(?Send)]
impl Perform for AddAdmin {
  type Response = AddAdminResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let added_admin = LocalUser::update(
      &mut context.pool(),
      data.local_user_id,
      &LocalUserUpdateForm::builder()
        .admin(Some(data.added))
        .build(),
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: added_admin.person_id,
      removed: Some(!data.added),
    };

    ModAdd::create(&mut context.pool(), &form).await?;

    let admins = PersonView::admins(&mut context.pool()).await?;

    Ok(AddAdminResponse { admins })
  }
}
