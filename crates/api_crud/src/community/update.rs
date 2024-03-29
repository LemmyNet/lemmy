use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, EditCommunity},
  context::LemmyContext,
  request::replace_image,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_mod_action,
    get_url_blocklist,
    local_site_to_slur_regex,
    process_markdown_opt,
    proxy_image_link_opt_api,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{Community, CommunityUpdateForm},
    local_site::LocalSite,
  },
  traits::Crud,
  utils::{diesel_option_overwrite, naive_now},
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
  let url_blocklist = get_url_blocklist(&context).await?;
  check_slurs_opt(&data.title, &slur_regex)?;
  let description =
    process_markdown_opt(&data.description, &slur_regex, &url_blocklist, &context).await?;
  is_valid_body_field(&data.description, false)?;
  let old_community = Community::read(&mut context.pool(), data.community_id).await?;
  replace_image(&data.icon, &old_community.icon, &context).await?;
  replace_image(&data.banner, &old_community.banner, &context).await?;

  let description = diesel_option_overwrite(description);
  let icon = proxy_image_link_opt_api(&data.icon, &context).await?;
  let banner = proxy_image_link_opt_api(&data.banner, &context).await?;

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
    title: data.title.clone(),
    description,
    icon,
    banner,
    nsfw: data.nsfw,
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
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
