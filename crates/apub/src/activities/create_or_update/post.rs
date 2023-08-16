use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::{
    activities::{create_or_update::page::CreateOrUpdatePage, CreateOrUpdateType},
    InCommunity,
  },
};
use activitypub_federation::{
  config::Data,
  kinds::public,
  protocol::verification::{verify_domains_match, verify_urls_match},
  traits::{ActivityHandler, Actor, Object},
};
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::{
  aggregates::structs::PostAggregates,
  newtypes::PersonId,
  source::{
    community::Community,
    person::Person,
    post::{Post, PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::error::{LemmyError, LemmyErrorType};
use url::Url;

impl CreateOrUpdatePage {
  pub(crate) async fn new(
    post: ApubPost,
    actor: &ApubPerson,
    community: &ApubCommunity,
    kind: CreateOrUpdateType,
    context: &Data<LemmyContext>,
  ) -> Result<CreateOrUpdatePage, LemmyError> {
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    Ok(CreateOrUpdatePage {
      actor: actor.id().into(),
      to: vec![public()],
      object: post.into_json(context).await?,
      cc: vec![community.id()],
      kind,
      id: id.clone(),
      audience: Some(community.id().into()),
    })
  }

  #[tracing::instrument(skip_all)]
  pub(crate) async fn send(
    post: Post,
    person_id: PersonId,
    kind: CreateOrUpdateType,
    context: Data<LemmyContext>,
  ) -> Result<(), LemmyError> {
    let post = ApubPost(post);
    let community_id = post.community_id;
    let person: ApubPerson = Person::read(&mut context.pool(), person_id).await?.into();
    let community: ApubCommunity = Community::read(&mut context.pool(), community_id)
      .await?
      .into();

    let create_or_update =
      CreateOrUpdatePage::new(post, &person, &community, kind, &context).await?;
    let is_mod_action = create_or_update.object.is_mod_action(&context).await?;
    let activity = AnnouncableActivities::CreateOrUpdatePost(create_or_update);
    send_activity_in_community(
      activity,
      &person,
      &community,
      vec![],
      is_mod_action,
      &context,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdatePage {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(&self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    insert_received_activity(&self.id, context).await?;
    verify_is_public(&self.to, &self.cc)?;
    let community = self.community(context).await?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_community_deleted_or_removed(&community)?;

    match self.kind {
      CreateOrUpdateType::Create => {
        verify_domains_match(self.actor.inner(), self.object.id.inner())?;
        verify_urls_match(self.actor.inner(), self.object.creator()?.inner())?;
        // Check that the post isnt locked, as that isnt possible for newly created posts.
        // However, when fetching a remote post we generate a new create activity with the current
        // locked value, so this check may fail. So only check if its a local community,
        // because then we will definitely receive all create and update activities separately.
        let is_locked = self.object.comments_enabled == Some(false);
        if community.local && is_locked {
          return Err(LemmyErrorType::NewPostCannotBeLocked)?;
        }
      }
      CreateOrUpdateType::Update => {
        let is_mod_action = self.object.is_mod_action(context).await?;
        if is_mod_action {
          verify_mod_action(&self.actor, self.object.id.inner(), community.id, context).await?;
        } else {
          verify_domains_match(self.actor.inner(), self.object.id.inner())?;
          verify_urls_match(self.actor.inner(), self.object.creator()?.inner())?;
        }
      }
    }
    ApubPost::verify(&self.object, self.actor.inner(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<LemmyContext>) -> Result<(), LemmyError> {
    let post = ApubPost::from_json(self.object, context).await?;

    // author likes their own post by default
    let like_form = PostLikeForm {
      post_id: post.id,
      person_id: post.creator_id,
      score: 1,
    };
    PostLike::like(&mut context.pool(), &like_form).await?;

    // Calculate initial hot_rank for post
    PostAggregates::update_hot_rank(&mut context.pool(), post.id).await?;

    Ok(())
  }
}
