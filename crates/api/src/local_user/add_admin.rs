use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  person::{AddAdmin, AddAdminResponse},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{
  source::{
    moderator::{ModAdd, ModAddForm},
    person::Person,
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::PersonViewSafe;
use lemmy_utils::{error::LemmyError, ConnectionId};
use lemmy_websocket::{messages::SendAllMessage, LemmyContext, UserOperation};

#[async_trait::async_trait(?Send)]
impl Perform for AddAdmin {
  type Response = AddAdminResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<AddAdminResponse, LemmyError> {
    let data: &AddAdmin = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let added = data.added;
    let added_person_id = data.person_id;
    let added_admin = blocking(context.pool(), move |conn| {
      Person::add_admin(conn, added_person_id, added)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_user"))?;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: added_admin.id,
      removed: Some(!data.added),
    };

    blocking(context.pool(), move |conn| ModAdd::create(conn, &form)).await??;

    let admins = blocking(context.pool(), PersonViewSafe::admins).await??;

    let res = AddAdminResponse { admins };

    context.chat_server().do_send(SendAllMessage {
      op: UserOperation::AddAdmin,
      response: res.clone(),
      websocket_id,
    });

    Ok(res)
  }
}
