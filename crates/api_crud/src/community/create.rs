use super::check_community_visibility_allowed;
use activitypub_federation::{config::Data, http_signatures::generate_actor_keypair};
use actix_web::web::Json;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, CreateCommunity},
  context::LemmyContext,
  utils::{
    generate_followers_url,
    generate_inbox_url,
    get_url_blocklist,
    is_admin,
    local_site_to_slur_regex,
    process_markdown_opt,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityFollowerState,
      CommunityInsertForm,
      CommunityModerator,
      CommunityModeratorForm,
    },
  },
  traits::{ApubActor, Crud, Followable, Joinable},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
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

  let slur_regex = local_site_to_slur_regex(&local_site);
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

  is_valid_actor_name(&data.name, local_site.actor_name_max_length as usize)?;

  if let Some(desc) = &data.description {
    is_valid_body_field(desc, false)?;
  }

  check_community_visibility_allowed(data.visibility, &local_user_view)?;

  // Double check for duplicate community actor_ids
  let community_ap_id = Community::local_url(&data.name, context.settings())?;
  let community_dupe = Community::read_from_apub_id(&mut context.pool(), &community_ap_id).await?;
  if community_dupe.is_some() {
    Err(LemmyErrorType::CommunityAlreadyExists)?
  }

  // When you create a community, make sure the user becomes a moderator and a follower
  let keypair = generate_actor_keypair()?;

  let community_form = CommunityInsertForm {
    sidebar,
    description,
    nsfw: data.nsfw,
    ap_id: Some(community_ap_id.clone()),
    private_key: Some(keypair.private_key),
    followers_url: Some(generate_followers_url(&community_ap_id)?),
    inbox_url: Some(generate_inbox_url()?),
    posting_restricted_to_mods: data.posting_restricted_to_mods,
    visibility: data.visibility,
    ..CommunityInsertForm::new(
      site_view.site.instance_id,
      data.name.clone(),
      data.title.clone(),
      keypair.public_key,
    )
  };

  let inserted_community = Community::create(&mut context.pool(), &community_form)
    .await
    .with_lemmy_type(LemmyErrorType::CommunityAlreadyExists)?;

  // The community creator becomes a moderator
  let community_moderator_form = CommunityModeratorForm {
    community_id: inserted_community.id,
    person_id: local_user_view.person.id,
  };

  CommunityModerator::join(&mut context.pool(), &community_moderator_form)
    .await
    .with_lemmy_type(LemmyErrorType::CommunityModeratorAlreadyExists)?;

  // Follow your own community
  let community_follower_form = CommunityFollowerForm {
    community_id: inserted_community.id,
    person_id: local_user_view.person.id,
    state: Some(CommunityFollowerState::Accepted),
    approver_id: None,
  };

  CommunityFollower::follow(&mut context.pool(), &community_follower_form)
    .await
    .with_lemmy_type(LemmyErrorType::CommunityFollowerAlreadyExists)?;

  // Update the discussion_languages if that's provided
  let community_id = inserted_community.id;
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

  build_community_response(&context, local_user_view, community_id).await
}
