use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{AddAdmin, AddAdminResponse},
  sensitive::Sensitive,
  utils::{is_admin, local_user_view_from_jwt_new},
  websocket::UserOperation,
};
use lemmy_db_schema::{
  source::{
    moderator::{ModAdd, ModAddForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{error::LemmyError, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for AddAdmin {
  type Response = AddAdminResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = self;
    let local_user_view = local_user_view_from_jwt_new(auth, context).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let added = data.added;
    let added_person_id = data.person_id;
    let added_admin = Person::update(
      context.pool(),
      added_person_id,
      &PersonUpdateForm::builder().admin(Some(added)).build(),
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

    let res = AddAdminResponse { admins };

    context.send_all_ws_message(&UserOperation::AddAdmin, &res, websocket_id)?;

    Ok(res)
  }
}
