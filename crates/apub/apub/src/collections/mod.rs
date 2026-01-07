use crate::{
  collections::community_moderators::handle_community_moderators,
  is_new_instance,
  protocol::collections::url_collection::UrlCollection,
};
use activitypub_federation::{
  actix_web::response::create_http_response,
  config::Data,
  fetch::{collection_id::CollectionId, object_id::ObjectId},
};
use actix_web::HttpResponse;
use community_featured::ApubCommunityFeatured;
use community_follower::ApubCommunityFollower;
use community_moderators::ApubCommunityModerators;
use community_outbox::ApubCommunityOutbox;
use lemmy_api_utils::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::group::Group,
  utils::protocol::{AttributedTo, PersonOrGroupType},
};
use lemmy_db_schema::source::{comment::Comment, post::Post};
use lemmy_utils::{FEDERATION_CONTEXT, error::LemmyResult, spawn_try_task};

pub(crate) mod community_featured;
pub(crate) mod community_follower;
pub(crate) mod community_moderators;
pub(crate) mod community_outbox;

pub fn fetch_community_collections(
  community: ApubCommunity,
  group: Group,
  context: Data<LemmyContext>,
) {
  spawn_try_task(async move {
    let outbox: CollectionId<ApubCommunityOutbox> = group.outbox.into();
    outbox.dereference(&community, &context).await.ok();
    if let Some(followers) = group.followers {
      let followers: CollectionId<ApubCommunityFollower> = followers.into();
      followers.dereference(&community, &context).await.ok();
    }
    // Dont fetch featured posts for new instances to save requests.
    // But need to run this in debug mode so that api tests can pass.
    if (cfg!(debug_assertions) || !is_new_instance(&context).await?)
      && let Some(featured) = group.featured
    {
      let featured: CollectionId<ApubCommunityFeatured> = featured.into();
      featured.dereference(&community, &context).await.ok();
    }
    if let Some(moderators) = group.attributed_to {
      if let AttributedTo::Lemmy(l) = moderators {
        let moderators: CollectionId<ApubCommunityModerators> = l.moderators().into();
        moderators.dereference(&community, &context).await.ok();
      } else if let AttributedTo::Peertube(p) = moderators {
        let new_mods = p
          .iter()
          .filter(|p| p.kind == PersonOrGroupType::Person)
          .map(|p| ObjectId::<ApubPerson>::from(p.id.clone().into_inner()))
          .collect();
        handle_community_moderators(&new_mods, &community, &context)
          .await
          .ok();
      }
    }
    Ok(())
  });
}

impl UrlCollection {
  pub(crate) async fn new_response(
    post: &Post,
    id: String,
    context: &LemmyContext,
  ) -> LemmyResult<HttpResponse> {
    let mut ordered_items = vec![post.ap_id.clone().into()];
    let comments = Comment::read_ap_ids_for_post(post.id, &mut context.pool()).await?;
    ordered_items.extend(comments.into_iter().map(Into::into));
    let collection = Self {
      r#type: Default::default(),
      id,
      total_items: ordered_items.len().try_into()?,
      ordered_items,
    };
    Ok(create_http_response(collection, &FEDERATION_CONTEXT)?)
  }

  /// Empty placeholder outbox used for Person, Instance, which dont implement a proper outbox.
  pub(crate) fn new_empty_response(id: String) -> LemmyResult<HttpResponse> {
    let collection = Self {
      r#type: Default::default(),
      id,
      ordered_items: vec![],
      total_items: 0,
    };
    Ok(create_http_response(collection, &FEDERATION_CONTEXT)?)
  }
}
