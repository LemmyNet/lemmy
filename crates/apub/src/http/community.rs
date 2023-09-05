use crate::{
    activity_lists::GroupInboxActivities,
    collections::{
        community_featured::ApubCommunityFeatured, community_moderators::ApubCommunityModerators,
        community_outbox::ApubCommunityOutbox,
    },
    http::{create_apub_response, create_apub_tombstone_response},
    objects::{community::ApubCommunity, person::ApubPerson},
    protocol::collections::group_followers::GroupFollowers,
};
use activitypub_federation::{
    actix_web::inbox::receive_activity,
    config::Data,
    protocol::context::WithContext,
    traits::{Collection, Object},
};
use actix_web::{web, web::Bytes, HttpRequest, HttpResponse};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{source::community::Community, traits::ApubActor};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct CommunityQuery {
    community_name: String,
}

/// Return the ActivityPub json representation of a local community over HTTP.
#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_http(
    info: web::Path<CommunityQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let community: ApubCommunity =
        Community::read_from_name(&mut context.pool(), &info.community_name, true)
            .await?
            .into();

    if !community.deleted && !community.removed {
        let apub = community.into_json(&context).await?;

        create_apub_response(&apub)
    } else {
        create_apub_tombstone_response(community.actor_id.clone())
    }
}

/// Handler for all incoming receive to community inboxes.
#[tracing::instrument(skip_all)]
pub async fn community_inbox(
    request: HttpRequest,
    body: Bytes,
    data: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    receive_activity::<WithContext<GroupInboxActivities>, ApubPerson, LemmyContext>(
        request, body, &data,
    )
    .await
}

/// Returns an empty followers collection, only populating the size (for privacy).
pub(crate) async fn get_apub_community_followers(
    info: web::Path<CommunityQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let community =
        Community::read_from_name(&mut context.pool(), &info.community_name, false).await?;
    let followers = GroupFollowers::new(community, &context).await?;
    create_apub_response(&followers)
}

/// Returns the community outbox, which is populated by a maximum of 20 posts (but no other
/// activites like votes or comments).
pub(crate) async fn get_apub_community_outbox(
    info: web::Path<CommunityQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let community: ApubCommunity =
        Community::read_from_name(&mut context.pool(), &info.community_name, false)
            .await?
            .into();
    if community.deleted || community.removed {
        Err(LemmyErrorType::Deleted)?
    }
    let outbox = ApubCommunityOutbox::read_local(&community, &context).await?;
    create_apub_response(&outbox)
}

#[tracing::instrument(skip_all)]
pub(crate) async fn get_apub_community_moderators(
    info: web::Path<CommunityQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let community: ApubCommunity =
        Community::read_from_name(&mut context.pool(), &info.community_name, false)
            .await?
            .into();
    if community.deleted || community.removed {
        Err(LemmyErrorType::Deleted)?
    }
    let moderators = ApubCommunityModerators::read_local(&community, &context).await?;
    create_apub_response(&moderators)
}

/// Returns collection of featured (stickied) posts.
pub(crate) async fn get_apub_community_featured(
    info: web::Path<CommunityQuery>,
    context: Data<LemmyContext>,
) -> Result<HttpResponse, LemmyError> {
    let community: ApubCommunity =
        Community::read_from_name(&mut context.pool(), &info.community_name, false)
            .await?
            .into();
    if community.deleted || community.removed {
        Err(LemmyErrorType::Deleted)?
    }
    let featured = ApubCommunityFeatured::read_local(&community, &context).await?;
    create_apub_response(&featured)
}
