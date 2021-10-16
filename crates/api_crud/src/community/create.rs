use crate::PerformCrud;
use actix_web::web::Data;
use lemmy_api_common::{
  blocking,
  community::{CommunityResponse, CreateCommunity},
  get_local_user_view_from_jwt,
  is_admin,
};
use lemmy_apub::{
  fetcher::object_id::ObjectId,
  generate_apub_endpoint,
  generate_followers_url,
  generate_inbox_url,
  generate_shared_inbox_url,
  EndpointType,
};
use lemmy_db_schema::{
  diesel_option_overwrite_to_url,
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
};
use lemmy_db_views_actor::community_view::CommunityView;
use lemmy_utils::{
  apub::generate_actor_keypair,
  utils::{check_slurs, check_slurs_opt, is_valid_actor_name},
  ApiError,
  ConnectionId,
  LemmyError,
};
use lemmy_websocket::LemmyContext;

#[async_trait::async_trait(?Send)]
impl PerformCrud for CreateCommunity {
  type Response = CommunityResponse;

  async fn perform(
    &self,
    context: &Data<LemmyContext>,
    _websocket_id: Option<ConnectionId>,
  ) -> Result<CommunityResponse, LemmyError> {
    let data: &CreateCommunity = self;
    let local_user_view =
      get_local_user_view_from_jwt(&data.auth, context.pool(), context.secret()).await?;

    let site = blocking(context.pool(), move |conn| Site::read(conn, 0)).await??;
    if site.community_creation_admin_only && is_admin(&local_user_view).is_err() {
      return Err(ApiError::err_plain("only_admins_can_create_communities").into());
    }

    check_slurs(&data.name, &context.settings().slur_regex())?;
    check_slurs(&data.title, &context.settings().slur_regex())?;
    check_slurs_opt(&data.description, &context.settings().slur_regex())?;

    if !is_valid_actor_name(&data.name, context.settings().actor_name_max_length) {
      return Err(ApiError::err_plain("invalid_community_name").into());
    }

    // Double check for duplicate community actor_ids
    let community_actor_id = generate_apub_endpoint(
      EndpointType::Community,
      &data.name,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let community_actor_id_wrapped = ObjectId::<Community>::new(community_actor_id.clone());
    let community_dupe = community_actor_id_wrapped.dereference_local(context).await;
    if community_dupe.is_ok() {
      return Err(ApiError::err_plain("community_already_exists").into());
    }

    // Check to make sure the icon and banners are urls
    let icon = diesel_option_overwrite_to_url(&data.icon)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;

    // When you create a community, make sure the user becomes a moderator and a follower
    let keypair = generate_actor_keypair()?;

    let community_form = CommunityForm {
      name: data.name.to_owned(),
      title: data.title.to_owned(),
      description: data.description.to_owned(),
      icon,
      banner,
      nsfw: data.nsfw,
      actor_id: Some(community_actor_id.to_owned()),
      private_key: Some(keypair.private_key),
      public_key: Some(keypair.public_key),
      followers_url: Some(generate_followers_url(&community_actor_id)?),
      inbox_url: Some(generate_inbox_url(&community_actor_id)?),
      shared_inbox_url: Some(Some(generate_shared_inbox_url(&community_actor_id)?)),
      ..CommunityForm::default()
    };

    let inserted_community = blocking(context.pool(), move |conn| {
      Community::create(conn, &community_form)
    })
    .await?
    .map_err(|e| ApiError::err("community_already_exists", e))?;

    // The community creator becomes a moderator
    let community_moderator_form = CommunityModeratorForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
    };

    let join = move |conn: &'_ _| CommunityModerator::join(conn, &community_moderator_form);
    if blocking(context.pool(), join).await?.is_err() {
      return Err(ApiError::err_plain("community_moderator_already_exists").into());
    }

    // Follow your own community
    let community_follower_form = CommunityFollowerForm {
      community_id: inserted_community.id,
      person_id: local_user_view.person.id,
      pending: false,
    };

    let follow = move |conn: &'_ _| CommunityFollower::follow(conn, &community_follower_form);
    if blocking(context.pool(), follow).await?.is_err() {
      return Err(ApiError::err_plain("community_follower_already_exists").into());
    }

    let person_id = local_user_view.person.id;
    let community_view = blocking(context.pool(), move |conn| {
      CommunityView::read(conn, inserted_community.id, Some(person_id))
    })
    .await??;

    Ok(CommunityResponse { community_view })
  }
}
