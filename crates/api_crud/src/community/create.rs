use crate::PerformCrud;
use activitypub_federation::core::{object_id::ObjectId, signatures::generate_actor_keypair};
use actix_web::web::Data;
use lemmy_api_common::{
  community::{CommunityResponse, CreateCommunity},
  utils::{blocking, get_local_user_view_from_jwt, is_admin},
};
use lemmy_apub::{
  generate_followers_url,
  generate_inbox_url,
  generate_local_apub_endpoint,
  generate_shared_inbox_url,
  objects::community::ApubCommunity,
  EndpointType,
};
use lemmy_db_schema::{
  source::{
    community::{
      Community,
      CommunityFollower,
      CommunityFollowerForm,
      CommunityForm,
      CommunityModerator,
      CommunityModeratorForm,
    },
    site::Site,
  },
  traits::{Crud, Followable, Joinable},
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url},
};
use lemmy_db_views_actor::structs::CommunityView;
use lemmy_utils::{
  error::LemmyError,
  utils::{check_slurs, check_slurs_opt, is_valid_actor_name},
  ConnectionId,
};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateCommunity {
  type Response = CommunityResponse;

  #[tracing::instrument(skip(context, _websocket_id))]
  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &CreateCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let site = blocking(context.pool(), Site::read_local_site).await??;
    if site.community_creation_admin_only && is_admin(&local_user_view).is_err() {
      return Err(LemmyError::from_message(
        "only_admins_can_create_communities",
      ));
    }

    // Check to make sure the icon and banners are urls
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;
    let description = diesel_option_overwrite(&data.description);

    check_slurs(&data.name, &context.settings().slur_regex())?;
    check_slurs(&data.title, &context.settings().slur_regex())?;
    check_slurs_opt(&data.description, &context.settings().slur_regex())?;

    if !is_valid_actor_name(&data.name, context.settings().actor_name_max_length) {
      return Err(LemmyError::from_message("invalid_community_name"));
    }

    // Double check for duplicate community actor_ids
    let community_actor_id = generate_local_apub_endpoint(
      EndpointType::Community,
      &data.name,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let community_actor_id_wrapped = ObjectId::<ApubCommunity>::new(community_actor_id.clone());
    let community_dupe = community_actor_id_wrapped.dereference_local(context).await;
    if community_dupe.is_ok() {
      return Err(LemmyError::from_message("community_already_exists"));
    }

    // When you create a community, make sure the user becomes a moderator and a follower
    let keypair = generate_actor_keypair()?;

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description,
      icon,
      banner,
      nsfw: data.nsfw,
      actor_id: Some(community_actor_id.to_owned()),
      private_key: Some(Some(keypair.private_key)),
      public_key: Some(keypair.public_key),
      followers_url: Some(generate_followers_url(&community_actor_id)?),
      inbox_url: Some(generate_inbox_url(&community_actor_id)?),
      shared_inbox_url: Some(Some(generate_shared_inbox_url(&community_actor_id)?)),
      posting_restricted_to_mods: data.posting_restricted_to_mods,
      ..CommunityForm::default()
    };

    let inserted_community = blocking(context.pool(), move |conn| {
      Community::create(conn, &community_form)
    })
    .await?
    .map_err(|e| LemmyError::from_error_message(e, "community_already_exists"))?;

    // The community creator becomes a moderator
    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
    };

    let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
    blocking(context.pool(), join)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "community_moderator_already_exists"))?;

    // Follow your own community
    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
    blocking(context.pool(), follow)
      .await?
      .map_err(|e| LemmyError::from_error_message(e, "community_follower_already_exists"))?;

    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, inserted_community.id, Some(person_id))
    })
    .await??;

    Ok(CommunityResponse { community_view })
  }
}
