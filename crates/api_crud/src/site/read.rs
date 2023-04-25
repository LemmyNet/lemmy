use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  sensitive::Sensitive,
  site::{GetSite, GetSiteResponse, MyUserInfo},
  utils::local_user_settings_view_from_jwt_opt,
  websocket::handlers::online_users::GetUsersOnline,
};
use lemmy_db_schema::source::{
  actor_language::{LocalUserLanguage, SiteLanguage},
  language::Language,
  tagline::Tagline,
};
use lemmy_db_views::structs::{CustomEmojiView, SiteView};
use lemmy_db_views_actor::structs::{
  CommunityBlockView,
  CommunityFollowerView,
  CommunityModeratorView,
  PersonBlockView,
  PersonView,
};
use lemmy_utils::{error::LemmyError, version, ConnectionId};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetSite {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    auth: Option<Sensitive<String>>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<GetSiteResponse, LemmyError> {
    let site_view = SiteView::read_local(context.pool()).await?;

    let admins = PersonView::admins(context.pool()).await?;

    let online = context.chat_server().send(GetUsersOnline).await?;

    // Build the local user
    let my_user =
      if let Some(local_user_view) = local_user_settings_view_from_jwt_opt(auth, context).await? {
        let person_id = local_user_view.person.id;
        let local_user_id = local_user_view.local_user.id;

        let follows = CommunityFollowerView::for_person(context.pool(), person_id)
          .await
          .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

        let person_id = local_user_view.person.id;
        let community_blocks = CommunityBlockView::for_person(context.pool(), person_id)
          .await
          .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

        let person_id = local_user_view.person.id;
        let person_blocks = PersonBlockView::for_person(context.pool(), person_id)
          .await
          .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

        let moderates = CommunityModeratorView::for_person(context.pool(), person_id)
          .await
          .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

        let discussion_languages = LocalUserLanguage::read(context.pool(), local_user_id)
          .await
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

    let all_languages = Language::read_all(context.pool()).await?;
    let discussion_languages = SiteLanguage::read_local_raw(context.pool()).await?;
    let taglines = Tagline::get_all(context.pool(), site_view.local_site.id).await?;
    let custom_emojis = CustomEmojiView::get_all(context.pool(), site_view.local_site.id).await?;

    Ok(GetSiteResponse {
      site_view,
      admins,
      online,
      version: version::VERSION.to_string(),
      my_user,
      all_languages,
      discussion_languages,
      taglines,
      custom_emojis,
    })
  }
}
