use super::check_community_visibility_allowed;
use activitypub_federation::config::Data;
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, CreateCommunity},
  context::LemmyContext,
  utils::{
    check_nsfw_allowed,
    generate_followers_url,
    generate_inbox_url,
    get_url_blocklist,
    is_admin,
    process_markdown_opt,
    slur_regex,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, LocalUserLanguage, SiteLanguage},
    community::{
      Community,
      CommunityActions,
      CommunityFollowerForm,
      CommunityInsertForm,
      CommunityModeratorForm,
    },
  },
  traits::{ApubActor, Crud, Followable, Joinable},
};
use lemmy_db_schema_file::enums::CommunityFollowerState;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_site::SiteView;
use lemmy_utils::{
  error::{LemmyErrorType, LemmyResult},
  utils::{
    slurs::check_slurs,
    validation::{
      is_valid_actor_name,
      is_valid_body_field,
      site_or_community_description_length_check,
    },
  },
};

pub async fn create_community(
  data: Json<CreateCommunity>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<CommunityResponse>> {
  let site_view = SiteView::read_local(&mut context.pool()).await?;
  let local_site = site_view.local_site;

  if local_site.community_creation_admin_only && is_admin(&local_user_view).is_err() {
    Err(LemmyErrorType::OnlyAdminsCanCreateCommunities)?
  }

  check_nsfw_allowed(data.nsfw, Some(&local_site))?;
  let slur_regex = slur_regex(&context).await?;
  let url_blocklist = get_url_blocklist(&context).await?;
  check_slurs(&data.name, &slur_regex)?;
  check_slurs(&data.title, &slur_regex)?;
  let sidebar = process_markdown_opt(&data.sidebar, &slur_regex, &url_blocklist, &context).await?;

  // Ensure that the sidebar has fewer than the max num characters...
  if let Some(sidebar) = &sidebar {
    is_valid_body_field(sidebar, false)?;
  }

  let description = data.description.clone();
  if let Some(desc) = &description {
    site_or_community_description_length_check(desc)?;
    check_slurs(desc, &slur_regex)?;
  }

  is_valid_actor_name(&data.name, local_site.actor_name_max_length)?;

  if let Some(desc) = &data.description {
    is_valid_body_field(desc, false)?;
  }

  check_community_visibility_allowed(data.visibility, &local_user_view)?;

  // Double check for duplicate community actor_ids
  let community_ap_id = Community::generate_local_actor_url(&data.name, context.settings())?;
  let community_dupe = Community::read_from_apub_id(&mut context.pool(), &community_ap_id).await?;
  if community_dupe.is_some() {
    Err(LemmyErrorType::CommunityAlreadyExists)?
  }

  let community_form = CommunityInsertForm {
    sidebar,
    description,
    nsfw: data.nsfw,
    ap_id: Some(community_ap_id.clone()),
    private_key: site_view.site.private_key,
    followers_url: Some(generate_followers_url(&community_ap_id)?),
    inbox_url: Some(generate_inbox_url()?),
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
    ..CommunityInsertForm::new(
      site_view.site.instance_id,
      data.name.clone(),
      data.title.clone(),
      site_view.site.public_key,
    )
  };

  let inserted_community = Community::create(&mut context.pool(), &community_form).await?;
  let community_id = inserted_community.id;

  // The community creator becomes a moderator
  let community_moderator_form =
    CommunityModeratorForm::new(community_id, local_user_view.person.id);

  CommunityActions::join(&mut context.pool(), &community_moderator_form).await?;

  // Follow your own community
  let community_follower_form = CommunityFollowerForm::new(
    community_id,
    local_user_view.person.id,
    CommunityFollowerState::Accepted,
  );

  CommunityActions::follow(&mut context.pool(), &community_follower_form).await?;

  // Update the discussion_languages if that's provided
  let site_languages = SiteLanguage::read_local_raw(&mut context.pool()).await?;
  let languages = if let Some(languages) = data.discussion_languages.clone() {
    // check that community languages are a subset of site languages
    // https://stackoverflow.com/a/64227550
    let is_subset = languages.iter().all(|item| site_languages.contains(item));
    if !is_subset {
      Err(LemmyErrorType::LanguageNotAllowed)?
    }
    languages
  } else {
    // Copy languages from creator
    LocalUserLanguage::read(&mut context.pool(), local_user_view.local_user.id)
      .await?
      .into_iter()
      .filter(|l| site_languages.contains(l))
      .collect()
  };
  CommunityLanguage::update(&mut context.pool(), languages, community_id).await?;

  build_community_response(&context, local_user_view, community_id).await
}
