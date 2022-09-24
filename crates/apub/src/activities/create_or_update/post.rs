use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::{announce::GetCommunity, send_activity_in_community},
    generate_activity_id,
    verify_is_public,
    verify_mod_action,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  objects::{community::ApubCommunity, person::ApubPerson, post::ApubPost},
  protocol::activities::{create_or_update::post::CreateOrUpdatePost, CreateOrUpdateType},
  ActorType,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, ApubObject},
  utils::{verify_domains_match, verify_urls_match},
};
use activitystreams_kinds::public;
use lemmy_api_common::utils::blocking;
use lemmy_db_schema::{
  source::{
    community::Community,
    post::{PostLike, PostLikeForm},
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::error::LemmyError;
use lemmy_websocket::{send::send_post_ws_message, LemmyContext, UserOperationCrud};
use url::Url;

impl CreateOrUpdatePost {
  pub(crate) async fn new(
    post: ApubPost,
    actor: &ApubPerson,
    community: &ApubCommunity,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<CreateOrUpdatePost, LemmyError> {
    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    Ok(CreateOrUpdatePost {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      object: post.into_apub(context).await?,
      cc: vec![community.actor_id()],
      kind,
      id: id.clone(),
      unparsed: Default::default(),
    })
  }

  #[tracing::instrument(skip_all)]
  pub async fn send(
    post: ApubPost,
    actor: &ApubPerson,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_id = post.community_id;
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();

    let create_or_update = CreateOrUpdatePost::new(post, actor, &community, kind, context).await?;
    let activity = AnnouncableActivities::CreateOrUpdatePost(Box::new(create_or_update));
    send_activity_in_community(activity, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdatePost {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    let community = self.get_community(context, request_counter).await?;
    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    check_community_deleted_or_removed(&community)?;

    match self.kind {
      CreateOrUpdateType::Create => {
        verify_domains_match(self.actor.inner(), self.object.id.inner())?;
        verify_urls_match(self.actor.inner(), self.object.creator()?.inner())?;
        // Check that the post isnt locked or stickied, as that isnt possible for newly created posts.
        // However, when fetching a remote post we generate a new create activity with the current
        // locked/stickied value, so this check may fail. So only check if its a local community,
        // because then we will definitely receive all create and update activities separately.
        let is_stickied_or_locked =
          self.object.stickied == Some(true) || self.object.comments_enabled == Some(false);
        if community.local && is_stickied_or_locked {
          return Err(LemmyError::from_message(
            "New post cannot be stickied or locked",
          ));
        }
      }
      CreateOrUpdateType::Update => {
        let is_mod_action = self.object.is_mod_action(context).await?;
        if is_mod_action {
          verify_mod_action(
            &self.actor,
            self.object.id.inner(),
            &community,
            context,
            request_counter,
          )
          .await?;
        } else {
          verify_domains_match(self.actor.inner(), self.object.id.inner())?;
          verify_urls_match(self.actor.inner(), self.object.creator()?.inner())?;
        }
      }
    }
    ApubPost::verify(&self.object, self.actor.inner(), context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let post = ApubPost::from_apub(self.object, context, request_counter).await?;

    // author likes their own post by default
    let like_form = PostLikeForm {
      post_id: post.id,
      person_id: post.creator_id,
      score: 1,
    };
    blocking(context.pool(), move |conn: &mut _| {
      PostLike::like(conn, &like_form)
    })
    .await??;

    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreatePost,
      CreateOrUpdateType::Update => UserOperationCrud::EditPost,
    };
    send_post_ws_message(post.id, notif_type, None, None, context).await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for CreateOrUpdatePost {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    self
      .object
      .extract_community(context, request_counter)
      .await
  }
}
