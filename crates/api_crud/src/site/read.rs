use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  sensitive::Sensitive,
  site::{GetSite, GetSiteResponse, MyUserInfo},
  utils::{check_user_valid, check_validator_time},
};
use lemmy_db_schema::{
  newtypes::LocalUserId,
  source::{
    actor_language::{LocalUserLanguage, SiteLanguage},
    language::Language,
    tagline::Tagline,
  },
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView, SiteView};
use lemmy_db_views_actor::structs::{
  CommunityBlockView,
  CommunityFollowerView,
  CommunityModeratorView,
  PersonBlockView,
  PersonView,
};
use lemmy_utils::{claims::Claims, error::LemmyError, version};

#[async_trait::async_trait(?Send)]
impl PerformCrud for GetSite {
  type Response = GetSiteResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<GetSiteResponse, LemmyError> {
    let mut conn = context.conn().await?;
    let data: &GetSite = self;

    let site_view = SiteView::read_local(&mut conn).await?;

    let admins = PersonView::admins(&mut conn).await?;

    // Build the local user
    let my_user = if let Some(local_user_view) =
      local_user_settings_view_from_jwt_opt(data.auth.as_ref(), context).await
    {
      let person_id = local_user_view.person.id;
      let local_user_id = local_user_view.local_user.id;

      let follows = CommunityFollowerView::for_person(&mut conn, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let person_id = local_user_view.person.id;
      let community_blocks = CommunityBlockView::for_person(&mut conn, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let person_id = local_user_view.person.id;
      let person_blocks = PersonBlockView::for_person(&mut conn, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let moderates = CommunityModeratorView::for_person(&mut conn, person_id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "system_err_login"))?;

      let discussion_languages = LocalUserLanguage::read(&mut conn, local_user_id)
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

    let all_languages = Language::read_all(&mut conn).await?;
    let discussion_languages = SiteLanguage::read_local_raw(&mut conn).await?;
    let taglines = Tagline::get_all(&mut conn, site_view.local_site.id).await?;
    let custom_emojis = CustomEmojiView::get_all(&mut conn, site_view.local_site.id).await?;

    Ok(GetSiteResponse {
      site_view,
      admins,
      version: version::VERSION.to_string(),
      my_user,
      all_languages,
      discussion_languages,
      taglines,
      custom_emojis,
    })
  }
}

#[tracing::instrument(skip_all)]
async fn local_user_settings_view_from_jwt_opt(
  jwt: Option<&Sensitive<String>>,
  context: &LemmyContext,
) -> Option<LocalUserView> {
  match jwt {
    Some(jwt) => {
      let claims = Claims::decode(jwt.as_ref(), &context.secret().jwt_secret)
        .ok()?
        .claims;
      let local_user_id = LocalUserId(claims.sub);
      let local_user_view = LocalUserView::read(&mut *context.conn().await.ok()?, local_user_id)
        .await
        .ok()?;
      check_user_valid(
        local_user_view.person.banned,
        local_user_view.person.ban_expires,
        local_user_view.person.deleted,
      )
      .ok()?;

      check_validator_time(&local_user_view.local_user.validator_time, &claims).ok()?;

      Some(local_user_view)
    }
    None => None,
  }
}
