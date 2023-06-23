use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{AddAdmin, AddAdminResponse},
  utils::{has_site_permission, local_user_view_from_jwt},
};
use lemmy_db_schema::{
  source::{
    local_site::LocalSite,
    moderator::{ModAdd, ModAddForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
  SitePermission,
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::error::LemmyError;

#[async_trait::async_trait(?Send)]
impl Perform for AddAdmin {
  type Response = AddAdminResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;

    let local_site = LocalSite::read(context.pool()).await?;

    // Make sure user is an admin
    has_site_permission(&local_user_view, SitePermission::AssignUserRoles)?;

    let added = data.added;
    let added_person_id = data.person_id;
    let added_admin = Person::update(
      context.pool(),
      added_person_id,
      &PersonUpdateForm::builder()
        .site_role_id(Some(if added {
          local_site.top_admin_role_id
        } else {
          local_site.default_site_role_id
        }))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_user"))?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: added_admin.id,
      removed: Some(!data.added),
    };

    ModAdd::create(context.pool(), &form).await?;

    let admins = PersonView::admins(context.pool()).await?;

    Ok(AddAdminResponse { admins })
  }
}
