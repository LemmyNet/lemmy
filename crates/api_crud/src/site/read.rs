use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  site::{GetSite, GetSiteResponse, MyUserInfo},
  utils::{blocking, build_federated_instances, get_local_user_settings_view_from_jwt_opt},
};
use lemmy_db_schema::source::{actor_language::SiteLanguage, language::Language};
use lemmy_db_views::structs::{LocalUserDiscussionLanguageView, SiteView};
use lemmy_db_views_actor::structs::{
  CommunityBlockView,
  CommunityFollowerView,
  CommunityModeratorView,
  PersonBlockView,
  PersonViewSafe,
};
use lemmy_utils::{error::LemmyError, version, ConnectionId};
use lemmy_websocket::{messages::GetUsersOnline, LemmyContext};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetSite {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &GetSite = self;

    let site_view = blocking(context.pool(), SiteView::read_local).await??;

    let admins = blocking(context.pool(), PersonViewSafe::admins).await??;

    let online = context
      .chat_server()
      .send(GetUsersOnline)
      .await
      .unwrap_or(1);

    // Build the local user
    let my_user = if let Some(local_user_view) = get_local_user_settings_view_from_jwt_opt(
      data.auth.as_ref(),
      context.pool(),
      context.secret(),
    )
    .await?
    {
      let person_id = local_user_view.person.id;
      let local_user_id = local_user_view.local_user.id;

      let follows = blocking(context.pool(), move |conn| {
        CommunityFollowerView::for_person(conn, person_id)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let person_id = local_user_view.person.id;
      let community_blocks = blocking(context.pool(), move |conn| {
        CommunityBlockView::for_person(conn, person_id)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let person_id = local_user_view.person.id;
      let person_blocks = blocking(context.pool(), move |conn| {
        PersonBlockView::for_person(conn, person_id)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let moderates = blocking(context.pool(), move |conn| {
        CommunityModeratorView::for_person(conn, person_id)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let discussion_languages = blocking(context.pool(), move |conn| {
        LocalUserDiscussionLanguageView::read_languages(conn, local_user_id)
      })
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      Some(MyUserInfo {
        local_user_view,
        follows,
        moderates,
        community_blocks,
        person_blocks,
        discussion_languages,
      })
    } else {
      None
    };

    let federated_instances =
      build_federated_instances(&site_view.local_site, context.pool()).await?;

    let all_languages = blocking(context.pool(), Language::read_all).await??;
    let discussion_languages = blocking(context.pool(), SiteLanguage::read_local).await??;

    Ok(GetSiteResponse {
      site_view,
      admins,
      online,
      version: version::VERSION.to_string(),
      my_user,
      federated_instances,
      all_languages,
      discussion_languages,
    })
  }
}
