use crate::{
  activities::{
    check_community_deleted_or_removed,
    community::send_activity_in_community,
    generate_activity_id,
    verify_is_public,
    verify_person_in_community,
  },
  activity_lists::AnnouncableActivities,
  insert_received_activity,
  mentions::MentionOrValue,
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
  protocol::{
    activities::{create_or_update::note::CreateOrUpdateNote, CreateOrUpdateType},
    InCommunity,
  },
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  kinds::public,
  protocol::verification::{verify_domains_match, verify_urls_match},
  traits::{ActivityHandler, Actor, Object},
};
use lemmy_api_common::{
  build_response::send_local_notifs,
  context::LemmyContext,
  utils::{check_post_deleted_or_removed, is_mod_or_admin},
};
use lemmy_db_schema::{
  aggregates::structs::CommentAggregates,
  newtypes::PersonId,
  source::{
    activity::ActivitySendTargets,
    comment::{Comment, CommentLike, CommentLikeForm},
    community::Community,
    person::Person,
    post::Post,
  },
  traits::{Crud, Likeable},
};
use lemmy_utils::{
  error::{LemmyError, LemmyResult},
  utils::mention::scrape_text_for_mentions,
  LemmyErrorType,
};
use url::Url;

impl CreateOrUpdateNote {
  #[tracing::instrument(skip(comment, person_id, kind, context))]
  pub(crate) async fn send(
    comment: Comment,
    person_id: PersonId,
    kind: CreateOrUpdateType,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    // TODO: might be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = Post::read(&mut context.pool(), post_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindPost)?;
    let community_id = post.community_id;
    let person: ApubPerson = Person::read(&mut context.pool(), person_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindPerson)?
      .into();
    let community: ApubCommunity = Community::read(&mut context.pool(), community_id)
      .await?
      .ok_or(LemmyErrorType::CouldntFindCommunity)?
      .into();

    let id = generate_activity_id(
      kind.clone(),
      &context.settings().get_protocol_and_hostname(),
    )?;
    let note = ApubComment(comment).into_json(&context).await?;

    let create_or_update = CreateOrUpdateNote {
      actor: person.id().into(),
      to: vec![public()],
      cc: note.cc.clone(),
      tag: note.tag.clone(),
      object: note,
      kind,
      id: id.clone(),
      audience: Some(community.id().into()),
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
      .map(ObjectId::from)
      .collect();
    let mut inboxes = ActivitySendTargets::empty();
    for t in tagged_users {
      let person = t.dereference(&context).await?;
      inboxes.add_inbox(person.shared_inbox_or_inbox());
    }

    let activity = AnnouncableActivities::CreateOrUpdateComment(create_or_update);
    send_activity_in_community(activity, &person, &community, inboxes, false, &context).await
  }
}

#[async_trait::async_trait]
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
  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    verify_is_public(&self.to, &self.cc)?;
    let post = self.object.get_parents(context).await?.0;
    let community = self.community(context).await?;

    verify_person_in_community(&self.actor, &community, context).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    check_community_deleted_or_removed(&community)?;
    check_post_deleted_or_removed(&post)?;
    verify_urls_match(self.actor.inner(), self.object.attributed_to.inner())?;

    ApubComment::verify(&self.object, self.actor.inner(), context).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all)]
  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    insert_received_activity(&self.id, context).await?;
    // Need to do this check here instead of Note::from_json because we need the person who
    // send the activity, not the comment author.
    let existing_comment = self.object.id.dereference_local(context).await.ok();
    if let (Some(distinguished), Some(existing_comment)) =
      (self.object.distinguished, existing_comment)
    {
      if distinguished != existing_comment.distinguished {
        let creator = self.actor.dereference(context).await?;
        let (post, _) = self.object.get_parents(context).await?;
        is_mod_or_admin(&mut context.pool(), &creator, post.community_id).await?;
      }
    }

    let comment = ApubComment::from_json(self.object, context).await?;

    // author likes their own comment by default
    let like_form = CommentLikeForm {
      comment_id: comment.id,
      post_id: comment.post_id,
      person_id: comment.creator_id,
      score: 1,
    };
    CommentLike::like(&mut context.pool(), &like_form).await?;

    // Calculate initial hot_rank
    CommentAggregates::update_hot_rank(&mut context.pool(), comment.id).await?;

    let do_send_email = self.kind == CreateOrUpdateType::Create;
    let actor = self.actor.dereference(context).await?;

    // Note:
    // Although mentions could be gotten from the post tags (they are included there), or the ccs,
    // Its much easier to scrape them from the comment body, since the API has to do that
    // anyway.
    // TODO: for compatibility with other projects, it would be much better to read this from cc or
    // tags
    let mentions = scrape_text_for_mentions(&comment.content);
    send_local_notifs(mentions, comment.id, &actor, do_send_email, context).await?;
    Ok(())
  }
}
