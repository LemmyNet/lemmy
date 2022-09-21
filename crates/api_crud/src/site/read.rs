use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  person::Register,
  site::{CreateSite, GetSite, GetSiteResponse, MyUserInfo},
  utils::{blocking, build_federated_instances, get_local_user_settings_view_from_jwt_opt},
};
use lemmy_db_schema::source::language::Language;
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
use tracing::info;

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetSite {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context, websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let data: &GetSite = self;

    let site_view = match blocking(context.pool(), SiteView::read_local).await? {
      Ok(site_view) => Some(site_view),
      // If the site isn't created yet, check the setup
      Err(_) => {
        if let Some(setup) = context.settings().setup.as_ref() {
          let register = Register {
            username: setup.admin_username.to_owned(),
            email: setup.admin_email.clone().map(|s| s.into()),
            password: setup.admin_password.clone().into(),
            password_verify: setup.admin_password.clone().into(),
            show_nsfw: true,
            captcha_uuid: None,
            captcha_answer: None,
            honeypot: None,
            answer: None,
          };
          let admin_jwt = register
            .perform(context, websocket_id)
            .await?
            .jwt
            .expect("jwt is returned from registration on newly created site");
          info!("Admin {} created", setup.admin_username);

          let create_site = CreateSite {
            name: setup.site_name.to_owned(),
            auth: admin_jwt,
            ..CreateSite::default()
          };
          create_site.perform(context, websocket_id).await?;
          info!("Site {} created", setup.site_name);
          Some(blocking(context.pool(), SiteView::read_local).await??)
        } else {
          None
        }
      }
    };

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

    let federated_instances = build_federated_instances(context.pool(), context.settings()).await?;

    let all_languages = blocking(context.pool(), Language::read_all).await??;

    Ok(GetSiteResponse {
      site_view,
      admins,
      online,
      version: version::VERSION.to_string(),
      my_user,
      federated_instances,
      all_languages,
    })
  }
}
