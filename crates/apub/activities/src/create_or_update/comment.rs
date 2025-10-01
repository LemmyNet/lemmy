use crate::{
  activity_lists::AnnouncableActivities,
  check_community_deleted_or_removed,
  community::send_activity_in_community,
  generate_activity_id,
  protocol::{create_or_update::note::CreateOrUpdateNote, CreateOrUpdateType},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  protocol::verification::{verify_domains_match, verify_urls_match},
  traits::{Activity, Actor, Object},
};
use lemmy_api_utils::{
  context::LemmyContext,
  notify::NotifyData,
  utils::{check_is_mod_or_admin, check_post_deleted_or_removed},
};
use lemmy_apub_objects::{
  objects::{comment::ApubComment, community::ApubCommunity, person::ApubPerson},
  utils::{
    functions::{generate_to, verify_person_in_community, verify_visibility},
    mentions::MentionOrValue,
    protocol::InCommunity,
  },
};
use lemmy_db_schema::{
  newtypes::PersonId,
  source::{
    activity::ActivitySendTargets,
    comment::{Comment, CommentActions, CommentLikeForm},
    community::Community,
    person::Person,
    post::Post,
  },
  traits::{Crud, Likeable},
};
use lemmy_db_views_site::SiteView;
use lemmy_utils::error::{LemmyError, LemmyResult};
use serde_json::{from_value, to_value};
use url::Url;

impl CreateOrUpdateNote {
  pub(crate) async fn send(
    comment: Comment,
    person_id: PersonId,
    kind: CreateOrUpdateType,
    context: Data<LemmyContext>,
  ) -> LemmyResult<()> {
    // TODO: might be helpful to add a comment method to retrieve community directly
    let post_id = comment.post_id;
    let post = Post::read(&mut context.pool(), post_id).await?;
    let community_id = post.community_id;
    let person: ApubPerson = Person::read(&mut context.pool(), person_id).await?.into();
    let community: ApubCommunity = Community::read(&mut context.pool(), community_id)
      .await?
      .into();

    let id = generate_activity_id(kind.clone(), &context)?;
    let note = ApubComment(comment).into_json(&context).await?;

    let create_or_update = CreateOrUpdateNote {
      actor: person.id().clone().into(),
      to: generate_to(&community)?,
      cc: note.cc.clone(),
      tag: note.tag.clone(),
      object: note,
      kind,
      id: id.clone(),
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

    // AnnouncableActivities doesnt contain Comment activity but only NoteWrapper,
    // to be able to handle both comment and private message. So to send this out we need
    // to convert this to NoteWrapper, by serializing and then deserializing again.
    let converted = from_value(to_value(create_or_update)?)?;
    let activity = AnnouncableActivities::CreateOrUpdateNoteWrapper(converted);
    send_activity_in_community(activity, &person, &community, inboxes, false, &context).await
  }
}

#[async_trait::async_trait]
impl Activity for CreateOrUpdateNote {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let post = self.object.get_parents(context).await?.0;
    let community = self.community(context).await?;
    verify_visibility(&self.to, &self.cc, &community)?;

    verify_person_in_community(&self.actor, &community, context).await?;
    verify_domains_match(self.actor.inner(), self.object.id.inner())?;
    check_community_deleted_or_removed(&community)?;
    check_post_deleted_or_removed(&post)?;
    verify_urls_match(self.actor.inner(), self.object.attributed_to.inner())?;

    ApubComment::verify(&self.object, self.actor.inner(), context).await?;
    Ok(())
  }

  async fn receive(self, context: &Data<Self::DataType>) -> LemmyResult<()> {
    let site_view = SiteView::read_local(&mut context.pool()).await?;

    // Need to do this check here instead of Note::from_json because we need the person who
    // send the activity, not the comment author.
    let existing_comment = self.object.id.dereference_local(context).await.ok();
    let (post, _) = self.object.get_parents(context).await?;
    if let (Some(distinguished), Some(existing_comment)) =
      (self.object.distinguished, existing_comment)
    {
      if distinguished != existing_comment.distinguished {
        let creator = self.actor.dereference(context).await?;
        check_is_mod_or_admin(&mut context.pool(), creator.id, post.community_id).await?;
      }
    }

    let comment = ApubComment::from_json(self.object, context).await?;

    // author likes their own comment by default
    let like_form = CommentLikeForm::new(comment.creator_id, comment.id, 1);
    CommentActions::like(&mut context.pool(), &like_form).await?;

    // Calculate initial hot_rank
    Comment::update_hot_rank(&mut context.pool(), comment.id).await?;

    let do_send_email =
      self.kind == CreateOrUpdateType::Create && !site_view.local_site.disable_email_notifications;
    let actor = self.actor.dereference(context).await?;

    // Note:
    // Although mentions could be gotten from the post tags (they are included there), or the ccs,
    // Its much easier to scrape them from the comment body, since the API has to do that
    // anyway.
    // TODO: for compatibility with other projects, it would be much better to read this from cc or
    // tags
    let community = Community::read(&mut context.pool(), post.community_id).await?;
    NotifyData::new(post.0, Some(comment.0), actor.0, community, do_send_email).send(context);
    Ok(())
  }
}
