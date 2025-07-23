use super::check_community_visibility_allowed;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use chrono::Utc;
use lemmy_api_utils::{
  build_response::build_community_response,
  context::LemmyContext,
  send_activity::{ActivityChannel, SendActivityData},
  utils::{
    check_community_mod_action,
    check_nsfw_allowed,
    get_url_blocklist,
    process_markdown_opt,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{Community, CommunityUpdateForm},
    mod_log::moderator::{ModChangeCommunityVisibility, ModChangeCommunityVisibilityForm},
  },
  traits::Crud,
  utils::diesel_string_update,
};
use lemmy_db_views_community::api::{CommunityResponse, EditCommunity};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::{slurs::check_slurs_opt, validation::is_valid_body_field},
};

pub async fn update_community(
  data: Json<EditCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  let local_site = SiteView::read_local(&mut context.pool()).await?.local_site;

  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  check_slurs_opt(&data.title, &slur_regex)?;
  check_nsfw_allowed(data.nsfw, Some(&local_site))?;

  let sidebar = diesel_string_update(
    process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context)
      .await?
      .as_deref(),
  );

  if let Some(Some(sidebar)) = &sidebar {
    is_valid_body_field(sidebar, false)?;
  }

  check_community_visibility_allowed(data.visibility, &local_user_view)?;
  let description = diesel_string_update(data.description.as_deref());

  let old_community = Community::read(&mut context.pool(), data.community_id).await?;

  // Verify its a mod (only mods can edit it)
  check_community_mod_action(&local_user_view, &old_community, false, &mut context.pool()).await?;

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
    sidebar,
    description,
    nsfw: data.nsfw,
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
    updated_at: Some(Some(Utc::now())),
    ..Default::default()
  };

  let community_id = data.community_id;
  let community = Community::update(&mut context.pool(), community_id, &community_form).await?;

  if old_community.visibility != community.visibility {
    let form = ModChangeCommunityVisibilityForm {
      mod_person_id: local_user_view.person.id,
      community_id: community.id,
      visibility: community.visibility,
    };
    ModChangeCommunityVisibility::create(&mut context.pool(), &form).await?;
  }

  ActivityChannel::submit_activity(
    SendActivityData::UpdateCommunity(local_user_view.person.clone(), community),
    &context,
  )?;

  build_community_response(&context, local_user_view, community_id).await
}
