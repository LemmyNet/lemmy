use crate::{
    activities::{
        community::send_activity_in_community, generate_activity_id, verify_is_public,
        verify_mod_action, verify_person_in_community,
    },
    activity_lists::AnnouncableActivities,
    insert_received_activity,
    objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
    protocol::{
        activities::community::{
            collection_add::CollectionAdd, collection_remove::CollectionRemove,
        },
        InCommunity,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::AddType, public},
    traits::{ActivityHandler, Actor},
};
use lemmy_api_common::{
    context::LemmyContext,
    utils::{generate_featured_url, generate_moderators_url},
};
use lemmy_db_schema::{
    impls::community::CollectionType,
    newtypes::{CommunityId, PersonId},
    source::{
        community::{Community, CommunityModerator, CommunityModeratorForm},
        moderator::{ModAddCommunity, ModAddCommunityForm},
        person::Person,
        post::{Post, PostUpdateForm},
    },
    traits::{Crud, Joinable},
};
use lemmy_utils::error::LemmyError;
use url::Url;

impl CollectionAdd {
    #[tracing::instrument(skip_all)]
    pub async fn send_add_mod(
        community: &ApubCommunity,
        added_mod: &ApubPerson,
        actor: &ApubPerson,
        context: &Data<LemmyContext>,
    ) -> Result<(), LemmyError> {
        let id = generate_activity_id(
            AddType::Add,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let add = CollectionAdd {
            actor: actor.id().into(),
            to: vec![public()],
            object: added_mod.id(),
            target: generate_moderators_url(&community.actor_id)?.into(),
            cc: vec![community.id()],
            kind: AddType::Add,
            id: id.clone(),
            audience: Some(community.id().into()),
        };

        let activity = AnnouncableActivities::CollectionAdd(add);
        let inboxes = vec![added_mod.shared_inbox_or_inbox()];
        send_activity_in_community(activity, actor, community, inboxes, true, context).await
    }

    pub async fn send_add_featured_post(
        community: &ApubCommunity,
        featured_post: &ApubPost,
        actor: &ApubPerson,
        context: &Data<LemmyContext>,
    ) -> Result<(), LemmyError> {
        let id = generate_activity_id(
            AddType::Add,
            &context.settings().get_protocol_and_hostname(),
        )?;
        let add = CollectionAdd {
            actor: actor.id().into(),
            to: vec![public()],
            object: featured_post.ap_id.clone().into(),
            target: generate_featured_url(&community.actor_id)?.into(),
            cc: vec![community.id()],
            kind: AddType::Add,
            id: id.clone(),
            audience: Some(community.id().into()),
        };
        let activity = AnnouncableActivities::CollectionAdd(add);
        send_activity_in_community(activity, actor, community, vec![], true, context).await
    }
}

#[async_trait::async_trait]
impl ActivityHandler for CollectionAdd {
    type DataType = LemmyContext;
    type Error = LemmyError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    #[tracing::instrument(skip_all)]
    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
        insert_received_activity(&self.id, context).await?;
        verify_is_public(&self.to, &self.cc)?;
        let community = self.community(context).await?;
        verify_person_in_community(&self.actor, &community, context).await?;
        verify_mod_action(&self.actor, &self.object, community.id, context).await?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), LemmyError> {
        let (community, collection_type) =
            Community::get_by_collection_url(&mut context.pool(), &self.target.into()).await?;
        match collection_type {
            CollectionType::Moderators => {
                let new_mod = ObjectId::<ApubPerson>::from(self.object)
                    .dereference(context)
                    .await?;

                // If we had to refetch the community while parsing the activity, then the new mod has already
                // been added. Skip it here as it would result in a duplicate key error.
                let new_mod_id = new_mod.id;
                let moderated_communities = CommunityModerator::get_person_moderated_communities(
                    &mut context.pool(),
                    new_mod_id,
                )
                .await?;
                if !moderated_communities.contains(&community.id) {
                    let form = CommunityModeratorForm {
                        community_id: community.id,
                        person_id: new_mod.id,
                    };
                    CommunityModerator::join(&mut context.pool(), &form).await?;

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
                // TODO: send websocket notification about added mod
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
) -> Result<(), LemmyError> {
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
) -> Result<(), LemmyError> {
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
