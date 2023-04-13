use crate::{
  activities::{
    deletion::{receive_delete_action, verify_delete_activity, DeletableObjects},
    generate_activity_id,
  },
  local_instance,
  objects::{community::ApubCommunity, person::ApubPerson},
  protocol::{activities::deletion::delete::Delete, IdOrNestedObject},
};
use activitypub_federation::{core::object_id::ObjectId, data::Data, traits::ActivityHandler};
use activitystreams_kinds::activity::DeleteType;
use lemmy_api_common::{context::LemmyContext, websocket::UserOperationCrud};
use lemmy_db_schema::{
  source::{
    comment::{Comment, CommentUpdateForm},
    community::{Community, CommunityUpdateForm},
    moderator::{
      ModRemoveComment,
      ModRemoveCommentForm,
      ModRemoveCommunity,
      ModRemoveCommunityForm,
      ModRemovePost,
      ModRemovePostForm,
    },
    post::{Post, PostUpdateForm},
  },
  traits::Crud,
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
impl ActivityHandler for Delete {
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
    verify_delete_activity(self, self.summary.is_some(), context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    if let Some(reason) = self.summary {
      // We set reason to empty string if it doesn't exist, to distinguish between delete and
      // remove. Here we change it back to option, so we don't write it to db.
      let reason = if reason.is_empty() {
        None
      } else {
        Some(reason)
      };
      receive_remove_action(
        &self
          .actor
          .dereference(context, local_instance(context).await, request_counter)
          .await?,
        self.object.id(),
        reason,
        context,
      )
      .await
    } else {
      receive_delete_action(
        self.object.id(),
        &self.actor,
        true,
        context,
        request_counter,
      )
      .await
    }
  }
}

impl Delete {
  pub(in crate::activities::deletion) fn new(
    actor: &ApubPerson,
    object: DeletableObjects,
    to: Url,
    community: Option<&Community>,
    summary: Option<String>,
    context: &LemmyContext,
  ) -> Result<Delete, LemmyError> {
    let id = generate_activity_id(
      DeleteType::Delete,
      &context.settings().get_protocol_and_hostname(),
    )?;
    let cc: Option<Url> = community.map(|c| c.actor_id.clone().into());
    Ok(Delete {
      actor: ObjectId::new(actor.actor_id.clone()),
      to: vec![to],
      object: IdOrNestedObject::Id(object.id()),
      cc: cc.into_iter().collect(),
      kind: DeleteType::Delete,
      summary,
      id,
      audience: community.map(|c| ObjectId::<ApubCommunity>::new(c.actor_id.clone())),
    })
  }
}

#[tracing::instrument(skip_all)]
pub(in crate::activities) async fn receive_remove_action(
  actor: &ApubPerson,
  object: &Url,
  reason: Option<String>,
  context: &LemmyContext,
) -> Result<(), LemmyError> {
  use UserOperationCrud::*;
  match DeletableObjects::read_from_db(object, context).await? {
    DeletableObjects::Community(community) => {
      if community.local {
        return Err(LemmyError::from_message(
          "Only local admin can remove community",
        ));
      }
      let form = ModRemoveCommunityForm {
        mod_person_id: actor.id,
        community_id: community.id,
        removed: Some(true),
        reason,
        expires: None,
      };
      ModRemoveCommunity::create(context.pool(), &form).await?;
      let deleted_community = Community::update(
        context.pool(),
        community.id,
        &CommunityUpdateForm::builder().removed(Some(true)).build(),
      )
      .await?;

      context
        .send_community_ws_message(&RemoveCommunity, deleted_community.id, None, None)
        .await?;
    }
    DeletableObjects::Post(post) => {
      let form = ModRemovePostForm {
        mod_person_id: actor.id,
        post_id: post.id,
        removed: Some(true),
        reason,
      };
      ModRemovePost::create(context.pool(), &form).await?;
      let removed_post = Post::update(
        context.pool(),
        post.id,
        &PostUpdateForm::builder().removed(Some(true)).build(),
      )
      .await?;

      context
        .send_post_ws_message(&RemovePost, removed_post.id, None, None)
        .await?;
    }
    DeletableObjects::Comment(comment) => {
      let form = ModRemoveCommentForm {
        mod_person_id: actor.id,
        comment_id: comment.id,
        removed: Some(true),
        reason,
      };
      ModRemoveComment::create(context.pool(), &form).await?;
      let removed_comment = Comment::update(
        context.pool(),
        comment.id,
        &CommentUpdateForm::builder().removed(Some(true)).build(),
      )
      .await?;

      context
        .send_comment_ws_message_simple(&RemoveComment, removed_comment.id)
        .await?;
    }
    DeletableObjects::PrivateMessage(_) => unimplemented!(),
  }
  Ok(())
}
