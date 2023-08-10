use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{AddAdmin, AddAdminResponse},
  utils::{is_admin, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModAdd, ModAddForm},
    person::{Person, PersonUpdateForm},
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

    let added = data.added;
    let added_person_id = data.person_id;
    let added_admin = Person::update(
      &mut context.pool(),
      added_person_id,
      &PersonUpdateForm {
        admin: Some(added),
        ..Default::default()
      },
    )
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateUser)?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: added_admin.id,
      removed: Some(!data.added),
    };

    ModAdd::create(&mut context.pool(), &form).await?;

    let admins = PersonView::admins(&mut context.pool()).await?;

    Ok(AddAdminResponse { admins })
  }
}
