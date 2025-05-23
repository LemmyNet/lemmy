use activitypub_federation::{
  config::Data,
  fetch::{collection_id::CollectionId, object_id::ObjectId},
};
use community_featured::ApubCommunityFeatured;
use community_follower::ApubCommunityFollower;
use community_moderators::ApubCommunityModerators;
use community_outbox::ApubCommunityOutbox;
use lemmy_api_common::context::LemmyContext;
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::group::Group,
  utils::{
    functions::handle_community_moderators,
    protocol::{AttributedTo, PersonOrGroupType},
  },
};
use lemmy_utils::spawn_try_task;

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
    if let Some(featured) = group.featured {
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
