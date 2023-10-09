use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, EditCommunity},
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{check_community_mod_action, local_site_to_slur_regex, sanitize_html_api_opt},
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{Community, CommunityUpdateForm},
    local_site::LocalSite,
  },
  traits::Crud,
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, naive_now},
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::{
  error::{LemmyError, LemmyErrorExt, LemmyErrorType},
  utils::{slurs::check_slurs_opt, validation::is_valid_body_field},
};

#[tracing::instrument(skip(context))]
pub async fn update_community(
  data: Json<EditCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> Result<Json<CommunityResponse>, LemmyError> {
  let local_site = LocalSite::read(&mut context.pool()).await?;

  let slur_regex = local_site_to_slur_regex(&local_site);
  check_slurs_opt(&data.title, &slur_regex)?;
  check_slurs_opt(&data.description, &slur_regex)?;
  is_valid_body_field(&data.description, false)?;

  let title = sanitize_html_api_opt(&data.title);
  let description = sanitize_html_api_opt(&data.description);

  let icon = diesel_option_overwrite_to_url(&data.icon)?;
  let banner = diesel_option_overwrite_to_url(&data.banner)?;
  let description = diesel_option_overwrite(description);

  // Verify its a mod (only mods can edit it)
  check_community_mod_action(
    &local_user_view.person,
    data.community_id,
    false,
    &mut context.pool(),
  )
  .await?;

  let community_id = data.community_id;
  if let Some(languages) = data.discussion_languages.clone() {
    let site_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
    // check that community languages are a subset of site languages
    // https://stackoverflow.com/a/64227550
    let is_subset = languages.iter().all(|item| site_languages.contains(item));
    if !is_subset {
      Err(LemmyErrorType::LanguageNotAllowed)?
    }
    CommunityLanguage::update(&mut context.pool(), languages, community_id).await?;
  }

  let community_form = CommunityUpdateForm {
    title,
    description,
    icon,
    banner,
    nsfw: data.nsfw,
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    updated: Some(Some(naive_now())),
    ..Default::default()
  };

  let community_id = data.community_id;
  let community = Community::update(&mut context.pool(), community_id, &community_form)
    .await
    .with_lemmy_type(LemmyErrorType::CouldntUpdateCommunity)?;

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )
  .await?;

  build_community_response(&context, local_user_view, community_id).await
}
