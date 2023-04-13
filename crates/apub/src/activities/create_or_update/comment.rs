use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::send_activity_in_community,
    create_or_update::get_comment_notif_recipients,
    generate_activity_id,
    verify_is_public,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  local_instance,
  mentions::MentionOrValue,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::{create_or_update::note::CreateOrUpdateNote, CreateOrUpdateType},
    InCommunity,
  },
  ActorType,
  SendActivity,
};
use activitypub_federation::{
  core::object_id::ObjectId,
  data::Data,
  traits::{ActivityHandler, Actor, ApubObject},
  utils::verify_domains_match,
};
use activitystreams_kinds::public;
use lemmy_api_common::{
  comment::{CommentResponse, CreateComment, EditComment},
  context::LemmyContext,
  utils::{check_post_deleted_or_removed, is_mod_or_admin},
  websocket::UserOperationCrud,
};
use lemmy_db_schema::{
  newtypes::PersonId,
  source::{
    comment::{Comment, CommentLike, CommentLikeForm},
    community::Community,
    person::Person,
    post::Post,
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::error::LemmyError;
use url::Url;

#[async_trait::async_trait(?Send)]
impl SendActivity for CreateComment {
  type Response = CommentResponse;

  async fn send_activity(
    _request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    CreateOrUpdateNote::send(
      &response.comment_view.comment,
      response.comment_view.creator.id,
      CreateOrUpdateType::Create,
      context,
    )
    .await
  }
}

#[async_trait::async_trait(?Send)]
impl SendActivity for EditComment {
  type Response = CommentResponse;

  async fn send_activity(
    _request: &Self,
    response: &Self::Response,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    CreateOrUpdateNote::send(
      &response.comment_view.comment,
      response.comment_view.creator.id,
      CreateOrUpdateType::Update,
      context,
    )
    .await
  }
}

impl CreateOrUpdateNote {
  #[tracing::instrument(skip(comment, person_id, kind, context))]
  async fn send(
    comment: &Comment,
    person_id: PersonId,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    // TODO: might be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = Post::read(context.pool(), post_id).await?;
    let community_id = post.community_id;
    let person: ApubPerson = Person::read(context.pool(), person_id).await?.into();
    let community: ApubCommunity = Community::read(context.pool(), community_id).await?.into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let note = ApubComment(comment.clone()).into_apub(context).await?;

    let create_or_update = CreateOrUpdateNote {
      actor: ObjectId::new(person.actor_id()),
      to: vec![public()],
      cc: note.cc.clone(),
      tag: note.tag.clone(),
      object: note,
      kind,
      id: id.clone(),
      audience: Some(ObjectId::new(community.actor_id())),
    };

    let tagged_users: Vec<ObjectId<ApubPerson>> = create_or_update
      .tag
      .iter()
      .filter_map(|t| {
        if let MentionOrValue::Mention(t) = t {
          Some(t)
        } else {
          None
        }
      })
      .map(|t| t.href.clone())
      .map(ObjectId::new)
      .collect();
    let mut inboxes = vec![];
    for t in tagged_users {
      let person = t
        .dereference(context, local_instance(context).await, &mut 0)
        .await?;
      inboxes.push(person.shared_inbox_or_inbox());
    }

    let activity = AnnouncableActivities::CreateOrUpdateComment(create_or_update);
    send_activity_in_community(activity, &person, &community, inboxes, false, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdateNote {
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
    let post = self.object.get_parents(context, request_counter).await?.0;
    let community = self.community(context, request_counter).await?;

    verify_person_in_community(&self.actor, &community, context, request_counter).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    check_community_deleted_or_removed(&community)?;
    check_post_deleted_or_removed(&post)?;

    ApubComment::verify(&self.object, self.actor.inner(), context, request_counter).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(
    self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // Need to do this check here instead of Note::from_apub because we need the person who
    // send the activity, not the comment author.
    let existing_comment = self.object.id.dereference_local(context).await.ok();
    if let (Some(distinguished), Some(existing_comment)) =
      (self.object.distinguished, existing_comment)
    {
      if distinguished != existing_comment.distinguished {
        let creator = self
          .actor
          .dereference(context, local_instance(context).await, request_counter)
          .await?;
        let (post, _) = self.object.get_parents(context, request_counter).await?;
        is_mod_or_admin(context.pool(), creator.id, post.community_id).await?;
      }
    }

    let comment = ApubComment::from_apub(self.object, context, request_counter).await?;

    // author likes their own comment by default
    let like_form = CommentLikeForm {
      comment_id: comment.id,
      post_id: comment.post_id,
      person_id: comment.creator_id,
      score: 1,
    };
    CommentLike::like(context.pool(), &like_form).await?;

    let do_send_email = self.kind == CreateOrUpdateType::Create;
    let recipients = get_comment_notif_recipients(
      &self.actor,
      &comment,
      do_send_email,
      context,
      request_counter,
    )
    .await?;
    let notif_type = match self.kind {
      CreateOrUpdateType::Create => UserOperationCrud::CreateComment,
      CreateOrUpdateType::Update => UserOperationCrud::EditComment,
    };
    context
      .send_comment_ws_message(&notif_type, comment.id, None, None, None, recipients)
      .await?;
    Ok(())
  }
}
