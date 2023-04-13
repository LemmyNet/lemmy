use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{BanPerson, BanPersonResponse},
  utils::{get_local_user_view_from_jwt, is_admin, remove_user_data},
  websocket::UserOperation,
};
use lemmy_db_schema::{
  source::{
    moderator::{ModBan, ModBanForm},
    person::{Person, PersonUpdateForm},
  },
  traits::Crud,
};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{error::LemmyError, utils::time::naive_from_unix, ConnectionId};

#[async_trait::async_trait(?Send)]
impl Perform for BanPerson {
  type Response = BanPersonResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<BanPersonResponse, LemmyError> {
    let data: &BanPerson = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    // Make sure user is an admin
    is_admin(&local_user_view)?;

    let ban = data.ban;
    let banned_person_id = data.person_id;
    let expires = data.expires.map(naive_from_unix);

    let person = Person::update(
      context.pool(),
      banned_person_id,
      &PersonUpdateForm::builder()
        .banned(Some(ban))
        .ban_expires(Some(expires))
        .build(),
    )
    .await
    .map_err(|e| LemmyError::from_error_message(e, "couldnt_update_user"))?;

    // Remove their data if that's desired
    let remove_data = data.remove_data.unwrap_or(false);
    if remove_data {
      remove_user_data(
        person.id,
        context.pool(),
        context.settings(),
        context.client(),
      )
      .await?;
    }

    // Mod tables
    let form = ModBanForm {
      mod_person_id: local_user_view.person.id,
      other_person_id: data.person_id,
      reason: data.reason.clone(),
      banned: Some(data.ban),
      expires,
    };

    ModBan::create(context.pool(), &form).await?;

    let person_id = data.person_id;
    let person_view = PersonView::read(context.pool(), person_id).await?;

    let res = BanPersonResponse {
      person_view,
      banned: data.ban,
    };

    context.send_all_ws_message(&UserOperation::BanPerson, &res, websocket_id)?;

    Ok(res)
  }
}
