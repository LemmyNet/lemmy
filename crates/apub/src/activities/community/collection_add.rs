use crate::{
  activities::{community::send_activity_in_community, generate_activity_id},
  activity_lists::AnnouncableActivities,
  protocol::activities::community::{
    collection_add::CollectionAdd,
    collection_remove::CollectionRemove,
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::activity::AddType,
  traits::{Activity, Actor, Object},
};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{generate_featured_url, generate_moderators_url},
};
use lemmy_apub_objects::{
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  utils::{
    functions::{generate_to, verify_mod_action, verify_person_in_community, verify_visibility},
    protocol::InCommunity,
  },
};
use lemmy_db_schema::{
  impls::community::CollectionType,
  newtypes::{CommunityId, PersonId},
  source::{
    activity::ActivitySendTargets,
    community::{Community, CommunityActions, CommunityModeratorForm},
    mod_log::moderator::{ModAddCommunity, ModAddCommunityForm},
    person::Person,
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl CollectionAdd {
  async fn send_add_mod(
    community: &ApubCommunity,
    added_mod: &ApubPerson,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let id = generate_activity_id(AddType::Add, context)?;
    let add = CollectionAdd {
      actor: actor.id().clone().into(),
      to: generate_to(community)?,
      object: added_mod.id().clone(),
      target: generate_moderators_url(&community.ap_id)?.into(),
      cc: vec![community.id().clone()],
      kind: AddType::Add,
      id: id.clone(),
    };

    let activity = AnnouncableActivities::CollectionAdd(add);
    let inboxes = ActivitySendTargets::to_inbox(added_mod.shared_inbox_or_inbox());
    send_activity_in_community(activity, actor, community, inboxes, true, context).await
  }

  async fn send_add_featured_post(
    community: &ApubCommunity,
    featured_post: &ApubPost,
    actor: &ApubPerson,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<()> {
    let id = generate_activity_id(AddType::Add, context)?;
    let add = CollectionAdd {
      actor: actor.id().clone().into(),
      to: generate_to(community)?,
      object: featured_post.ap_id.clone().into(),
      target: generate_featured_url(&community.ap_id)?.into(),
      cc: vec![community.id().clone()],
      kind: AddType::Add,
      id: id.clone(),
    };
    let activity = AnnouncableActivities::CollectionAdd(add);
    send_activity_in_community(
      activity,
      actor,
      community,
      ActivitySendTargets::empty(),
      true,
      context,
    )
    .await
  }
}

#[async_trait::async_trait]
impl Activity for CollectionAdd {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let community = self.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;
    verify_person_in_community(&self.actor, &community, context).await?;
    verify_mod_action(&self.actor, &community, context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let (community, collection_type) =
      Community::get_by_collection_url(&mut context.pool(), &self.target.into()).await?;
    match collection_type {
      CollectionType::Moderators => {
        let new_mod = ObjectId::<ApubPerson>::from(self.object)
          .dereference(context)
          .await?;

        // If we had to refetch the community while parsing the activity, then the new mod has
        // already been added. Skip it here as it would result in a duplicate key error.
        let new_mod_id = new_mod.id;
        let moderated_communities =
          CommunityActions::get_person_moderated_communities(&mut context.pool(), new_mod_id)
            .await?;
        if !moderated_communities.contains(&community.id) {
          let form = CommunityModeratorForm::new(community.id, new_mod.id);
          CommunityActions::join(&mut context.pool(), &form).await?;

          // write mod log
          let actor = self.actor.dereference(context).await?;
          let form = ModAddCommunityForm {
            mod_person_id: actor.id,
            other_person_id: new_mod.id,
            community_id: community.id,
            removed: Some(false),
          };
          ModAddCommunity::create(&mut context.pool(), &form).await?;
        }
      }
      CollectionType::Featured => {
        let post = ObjectId::<ApubPost>::from(self.object)
          .dereference(context)
          .await?;
        let form = PostUpdateForm {
          featured_community: Some(true),
          ..Default::default()
        };
        Post::update(&mut context.pool(), post.id, &form).await?;
      }
    }
    Ok(())
  }
}

pub(crate) async fn send_add_mod_to_community(
  actor: Person,
  community_id: CommunityId,
  updated_mod_id: PersonId,
  added: bool,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let actor: ApubPerson = actor.into();
  let community: ApubCommunity = Community::read(&mut context.pool(), community_id)
    .await?
    .into();
  let updated_mod: ApubPerson = Person::read(&mut context.pool(), updated_mod_id)
    .await?
    .into();
  if added {
    CollectionAdd::send_add_mod(&community, &updated_mod, &actor, &context).await
  } else {
    CollectionRemove::send_remove_mod(&community, &updated_mod, &actor, &context).await
  }
}

pub(crate) async fn send_feature_post(
  post: Post,
  actor: Person,
  featured: bool,
  context: Data<LemmyContext>,
) -> LemmyResult<()> {
  let actor: ApubPerson = actor.into();
  let post: ApubPost = post.into();
  let community = Community::read(&mut context.pool(), post.community_id)
    .await?
    .into();
  if featured {
    CollectionAdd::send_add_featured_post(&community, &post, &actor, &context).await
  } else {
    CollectionRemove::send_remove_featured_post(&community, &post, &actor, &context).await
  }
}
