use crate::PerformCrud;
use activitypub_federation::http_signatures::generate_actor_keypair;
use actix_web::web::Data;
use lemmy_api_common::{
  build_response::build_community_response,
  community::{CommunityResponse, CreateCommunity},
  context::LemmyContext,
  utils::{
    generate_followers_url,
    generate_inbox_url,
    generate_local_apub_endpoint,
    generate_shared_inbox_url,
    is_admin,
    local_site_to_slur_regex,
    local_user_view_from_jwt,
    EndpointType,
  },
};
use lemmy_db_schema::{
  source::{
    actor_language::{CommunityLanguage, SiteLanguage},
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityInsertForm,
      CommunityModerator,
      CommunityModeratorForm,
    },
  },
  traits::{ApubActor, Crud, Followable, Joinable},
  utils::diesel_option_overwrite_to_url_create,
};
use lemmy_db_views::structs::SiteView;
use lemmy_utils::{
  error::LemmyError,
  utils::{
    slurs::{check_slurs, check_slurs_opt},
    validation::{is_valid_actor_name, is_valid_body_field},
  },
};

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<CommunityResponse, LemmyError> {
    let data: &CreateCommunity = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let site_view = SiteView::read_local(context.pool()).await?;
    let local_site = site_view.local_site;

    if local_site.community_creation_admin_only && is_admin(&local_user_view).is_err() {
      return Err(LemmyError::from_message(
        "only_admins_can_create_communities",
      ));
    }

    // Check to make sure the icon and banners are urls
    let icon = diesel_option_overwrite_to_url_create(&data.icon)?;
    let banner = diesel_option_overwrite_to_url_create(&data.banner)?;

    let slur_regex = local_site_to_slur_regex(&local_site);
    check_slurs(&data.name, &slur_regex)?;
    check_slurs(&data.title, &slur_regex)?;
    check_slurs_opt(&data.description, &slur_regex)?;

    is_valid_actor_name(&data.name, local_site.actor_name_max_length as usize)?;
    is_valid_body_field(&data.description)?;

    // Double check for duplicate community actor_ids
    let community_actor_id = generate_local_apub_endpoint(
      EndpointType::Community,
      &data.name,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let community_dupe = Community::read_from_apub_id(context.pool(), &community_actor_id).await?;
    if community_dupe.is_some() {
      return Err(LemmyError::from_message("community_already_exists"));
    }

    // When you create a community, make sure the user becomes a moderator and a follower
    let keypair = generate_actor_keypair()?;

    let community_form = CommunityInsertForm::builder()
      .name(data.name.clone())
      .title(data.title.clone())
      .description(data.description.clone())
      .icon(icon)
      .banner(banner)
      .nsfw(data.nsfw)
      .actor_id(Some(community_actor_id.clone()))
      .private_key(Some(keypair.private_key))
      .public_key(keypair.public_key)
      .followers_url(Some(generate_followers_url(&community_actor_id)?))
      .inbox_url(Some(generate_inbox_url(&community_actor_id)?))
      .shared_inbox_url(Some(generate_shared_inbox_url(&community_actor_id)?))
      .posting_restricted_to_mods(data.posting_restricted_to_mods)
      .instance_id(site_view.site.instance_id)
      .build();

    let inserted_community = Community::create(context.pool(), &community_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "community_already_exists"))?;

    // The community creator becomes a moderator
    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
    };

    CommunityModerator::join(context.pool(), &community_moderator_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "community_moderator_already_exists"))?;

    // Follow your own community
    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    CommunityFollower::follow(context.pool(), &community_follower_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;

    // Update the discussion_languages if that's provided
    let community_id = inserted_community.id;
    if let Some(languages) = data.discussion_languages.clone() {
      let site_languages = SiteLanguage::read_local_raw(context.pool()).await?;
      // check that community languages are a subset of site languages
      // https://stackoverflow.com/a/64227550
      let is_subset = languages.iter().all(|item| site_languages.contains(item));
      if !is_subset {
        return Err(LemmyError::from_message("language_not_allowed"));
      }
      CommunityLanguage::update(context.pool(), languages, community_id).await?;
    }

    build_community_response(context, local_user_view, community_id).await
  }
}
