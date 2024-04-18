use actix_web::web::{Data, Json};
use lemmy_api_common::{context::LemmyContext, site::GetSiteResponse, utils::is_admin};
use lemmy_db_schema::{
  source::{
    actor_language::SiteLanguage,
    language::Language,
    local_site_url_blocklist::LocalSiteUrlBlocklist,
    local_user::{LocalUser, LocalUserUpdateForm},
    moderator::{ModAdd, ModAddForm},
    tagline::Tagline,
  },
  traits::Crud,
};
use lemmy_db_views::structs::{CustomEmojiView, LocalUserView, SiteView};
use lemmy_db_views_actor::structs::PersonView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  VERSION,
};

#[tracing::instrument(skip(context))]
pub async fn leave_admin(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetSiteResponse>> {
  is_admin(&local_user_view)?;

  // Make sure there isn't just one admin (so if one leaves, there will still be one left)
  let admins = PersonView::admins(&mut context.pool()).await?;
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
  let site_view = SiteView::read_local(&mut context.pool())
    .await?
    .ok_or(LemmyErrorType::LocalSiteNotSetup)?;
  let admins = PersonView::admins(&mut context.pool()).await?;

  let all_languages = Language::read_all(&mut context.pool()).await?;
  let discussion_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let taglines = Tagline::get_all(&mut context.pool(), site_view.local_site.id).await?;
  let custom_emojis =
    CustomEmojiView::get_all(&mut context.pool(), site_view.local_site.id).await?;
  let blocked_urls = LocalSiteUrlBlocklist::get_all(&mut context.pool()).await?;

  Ok(Json(GetSiteResponse {
    site_view,
    admins,
    version: VERSION.to_string(),
    my_user: None,
    all_languages,
    discussion_languages,
    taglines,
    custom_emojis,
    blocked_urls,
  }))
}
