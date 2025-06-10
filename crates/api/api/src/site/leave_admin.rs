use actix_web::web::{Data, Json};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  source::{
    actor_language::SiteLanguage,
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    local_user::{LocalUser, LocalUserUpdateForm},
    mod_log::moderator::{ModAdd, ModAddForm},
    oauth_provider::OAuthProvider,
    tagline::Tagline,
  },
  traits::Crud,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_person::impls::PersonQuery;
use lemmy_db_views_site::{api::GetSiteResponse, SiteView};
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  VERSION,
};

pub async fn leave_admin(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetSiteResponse>> {
  is_admin(&local_user_view)?;

  // Make sure there isn't just one admin (so if one leaves, there will still be one left)
  let admins = PersonQuery {
    admins_only: Some(true),
    ..Default::default()
  }
  .list(local_user_view.person.instance_id, &mut context.pool())
  .await?;
  if admins.len() == 1 {
    Err(LemmyErrorType::CannotLeaveAdmin)?
  }

  LocalUser::update(
    &mut context.pool(),
    local_user_view.local_user.id,
    &LocalUserUpdateForm {
      admin: Some(false),
      // Necessary because admins can bypass the registration applications (if they're turned on)
      // but then won't be able to log in because they haven't been approved.
      accepted_application: Some(true),
      ..Default::default()
    },
  )
  .await?;

  // Mod tables
  let person_id = local_user_view.person.id;
  let form = ModAddForm {
    mod_person_id: person_id,
    other_person_id: person_id,
    removed: Some(true),
  };

  ModAdd::create(&mut context.pool(), &form).await?;

  // Reread site and admins
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let admins = PersonQuery {
    admins_only: Some(true),
    ..Default::default()
  }
  .list(site_view.instance.id, &mut context.pool())
  .await?;

  let all_languages = Language::read_all(&mut context.pool()).await?;
  let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let oauth_providers = OAuthProvider::get_all_public(&mut context.pool()).await?;
  let blocked_urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;
  let tagline = Tagline::get_random(&mut context.pool()).await.ok();

  Ok(Json(GetSiteResponse {
    site_view,
    admins,
    version: VERSION.to_string(),
    all_languages,
    discussion_languages,
    oauth_providers,
    admin_oauth_providers: vec![],
    blocked_urls,
    tagline,
    my_user: None,
    image_upload_disabled: context.settings().pictrs()?.image_upload_disabled,
    active_plugins: vec![],
  }))
}
