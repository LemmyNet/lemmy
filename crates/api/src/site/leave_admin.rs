use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{GetSiteResponse, LeaveAdmin},
  utils::{blocking, build_federated_instances, get_local_user_view_from_jwt, is_admin},
};
use lemmy_db_schema::{
  source::{
    language::Language,
    moderator::{ModAdd, ModAddForm},
    person::Person,
  },
  traits::Crud,
};
use lemmy_db_views::structs::SiteView;
use lemmy_db_views_actor::structs::PersonViewSafe;
use lemmy_utils::{error::LemmyError, version, ConnectionId};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl Perform for LeaveAdmin {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &LeaveAdmin = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    is_admin(&local_user_view)?;

    // Make sure there isn't just one admin (so if one leaves, there will still be one left)
    let admins = blocking(context.pool(), PersonViewSafe::admins).await??;
    if admins.len() == 1 {
      return Err(LemmyError::from_message("cannot_leave_admin"));
    }

    let person_id = local_user_view.person.id;
    blocking(context.pool(), move |conn| {
      Person::leave_admin(conn, person_id)
    })
    .await??;

    // Mod tables
    let form = ModAddForm {
      mod_person_id: person_id,
      other_person_id: person_id,
      removed: Some(true),
    };

    blocking(context.pool(), move |conn| ModAdd::create(conn, &form)).await??;

    // Reread site and admins
    let site_view = blocking(context.pool(), SiteView::read_local).await??;
    let admins = blocking(context.pool(), PersonViewSafe::admins).await??;

    let federated_instances = build_federated_instances(context.pool(), context.settings()).await?;

    let all_languages = blocking(context.pool(), Language::read_all).await??;

    Ok(GetSiteResponse {
      site_view: Some(site_view),
      admins,
      online: 0,
      version: version::VERSION.to_string(),
      my_user: None,
      federated_instances,
      all_languages,
    })
  }
}
