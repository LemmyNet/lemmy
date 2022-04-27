use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::{announce::GetCommunity, send_activity_in_community},
    create_or_update::get_comment_notif_recipients,
    generate_activity_id,
    verify_activity,
    verify_is_public,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  mentions::MentionOrValue,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
  protocol::activities::{create_or_update::comment::CreateOrUpdateComment, CreateOrUpdateType},
};
use activitystreams_kinds::public;
use lemmy_api_common::{blocking, check_post_deleted_or_removed};
use lemmy_apub_lib::{
  data::Data,
  object_id::ObjectId,
  traits::{ActivityHandler, ActorType, ApubObject},
  verify::verify_domains_match,
};
use lemmy_db_schema::{
  source::{
    comment::{CommentLike, CommentLikeForm},
    community::Community,
    post::Post,
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::LemmyError;
use lemmy_websocket::{send::send_comment_ws_message, LemmyContext, UserOperationCrud};

impl CreateOrUpdateComment {
  #[tracing::instrument(skip(comment, actor, kind, context))]
  pub async fn send(
    comment: ApubComment,
    actor: &ApubPerson,
    kind: CreateOrUpdateType,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    // TODO: might be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = blocking(context.pool(), move |conn| Post::read(conn, post_id)).await??;
    let community_id = post.community_id;
    let community: ApubCommunity = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??
    .into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let note = comment.into_apub(context).await?;

    let create_or_update = CreateOrUpdateComment {
      actor: ObjectId::new(actor.actor_id()),
      to: vec![public()],
      cc: note.cc.clone(),
      tag: note.tag.clone(),
      object: note,
      kind,
      id: id.clone(),
      unparsed: Default::default(),
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
        .dereference(context, context.client(), request_counter)
        .await?;
      inboxes.push(person.shared_inbox_or_inbox_url());
    }

    let activity = AnnouncableActivities::CreateOrUpdateComment(create_or_update);
    send_activity_in_community(activity, &id, actor, &community, inboxes, context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for CreateOrUpdateComment {
  type DataType = LemmyContext;

  #[tracing::instrument(skip_all)]
  async fn verify(
    &self,
    context: &Data<LemmyContext>,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_is_public(&self.to, &self.cc)?;
    let post = self.object.get_parents(context, request_counter).await?.0;
    let community = self.get_community(context, request_counter).await?;

    verify_activity(&self.id, self.actor.inner(), &context.settings())?;
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
    let comment = ApubComment::from_apub(self.object, context, request_counter).await?;

    // author likes their own comment by default
    let like_form = CommentLikeForm {
      comment_id: comment.id,
      post_id: comment.post_id,
      person_id: comment.creator_id,
      score: 1,
    };
    blocking(context.pool(), move |conn: &'_ _| {
      CommentLike::like(conn, &like_form)
    })
    .await??;

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
    send_comment_ws_message(
      comment.id, notif_type, None, None, None, recipients, context,
    )
    .await?;
    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl GetCommunity for CreateOrUpdateComment {
  #[tracing::instrument(skip_all)]
  async fn get_community(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<ApubCommunity, LemmyError> {
    let post = self.object.get_parents(context, request_counter).await?.0;
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, post.community_id)
    })
    .await??;
    Ok(community.into())
  }
}
