use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, EditCommunity},
  context::LemmyContext,
  utils::{local_site_to_slur_regex, local_user_view_from_jwt, sanitize_html_opt},
};
use lemmy_db_schema::{
  newtypes::PersonId,
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{Community, CommunityUpdateForm},
    local_site::LocalSite,
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
};
use lemmy_db_views_actor::structs::CommunityModeratorView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{slurs::check_slurs_opt, validation::is_valid_body_field},
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for EditCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommunityResponse, LemmyError> {
    let data: &EditCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let local_site = LocalSite::read(&mut context.pool()).await?;

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs_opt(&data.title, &slur_regex)?;
    check_slurs_opt(&data.description, &slur_regex)?;
    is_valid_body_field(&data.description, false)?;

    let title = sanitize_html_opt(&data.title);
    let description = sanitize_html_opt(&data.description);

    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;
    let description = diesel_option_overwrite(description);

    // Verify its a mod (only mods can edit it)
    let community_id = data.community_id;
    let mods: Vec<PersonId> =
      CommunityModeratorView::for_community(&mut context.pool(), community_id)
        .await
        .map(|v| v.into_iter().map(|m| m.moderator.id).collect())?;
    if !mods.contains(&local_user_view.person.id) {
      return Err(LemmyErrorType::NotAModerator)?;
    }

    let community_id = data.community_id;
    if let Some(languages) = data.discussion_languages.clone() {
      let site_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
      // check that community languages are a subset of site languages
      // https://stackoverflow.com/a/64227550
      let is_subset = languages.iter().all(|item| site_languages.contains(item));
      if !is_subset {
        return Err(LemmyErrorType::LanguageNotAllowed)?;
      }
      CommunityLanguage::update(&mut context.pool(), languages, community_id).await?;
    }

    let community_form = CommunityUpdateForm::builder()
      .title(title)
      .description(description)
      .icon(icon)
      .banner(banner)
      .nsfw(data.nsfw)
      .posting_restricted_to_mods(data.posting_restricted_to_mods)
      .updated(Some(Some(naive_now())))
      .build();

    let community_id = data.community_id;
    Community::update(&mut context.pool(), community_id, &community_form)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunity)?;

    build_community_response(context, local_user_view, community_id).await
  }
}
