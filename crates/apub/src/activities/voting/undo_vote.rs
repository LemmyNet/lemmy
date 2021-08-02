use crate::{
  activities::{
    community::announce::AnnouncableActivities,
    generate_activity_id,
    verify_activity,
    verify_person_in_community,
    voting::{
      undo_vote_comment,
      undo_vote_post,
      vote::{Vote, VoteType},
    },
  },
  activity_queue::send_to_community_new,
  extensions::context::lemmy_context,
  fetcher::{
    objects::get_or_fetch_and_insert_post_or_comment,
    person::get_or_fetch_and_upsert_person,
  },
  ActorType,
  PostOrComment,
};
use activitystreams::activity::kind::UndoType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{values::PublicUrl, verify_urls_match, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::Crud;
use lemmy_db_schema::{
  source::{community::Community, person::Person},
  CommunityId,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use std::ops::Deref;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoVote {
  to: PublicUrl,
  object: Vote,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: UndoType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

impl UndoVote {
  pub async fn send(
    object: &PostOrComment,
    actor: &Person,
    community_id: CommunityId,
    kind: VoteType,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community = blocking(context.pool(), move |conn| {
      Community::read(conn, community_id)
    })
    .await??;
    let id = generate_activity_id(UndoType::Undo)?;

    let undo_vote = UndoVote {
      to: PublicUrl::Public,
      object: Vote {
        to: PublicUrl::Public,
        object: object.ap_id(),
        cc: [community.actor_id()],
        kind: kind.clone(),
        common: ActivityCommonFields {
          context: lemmy_context(),
          id: generate_activity_id(kind)?,
          actor: actor.actor_id(),
          unparsed: Default::default(),
        },
      },
      cc: [community.actor_id()],
      kind: UndoType::Undo,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };
    let activity = AnnouncableActivities::UndoVote(undo_vote);
    send_to_community_new(activity, &id, actor, &community, vec![], context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for UndoVote {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc[0], context, request_counter).await?;
    verify_urls_match(&self.common.actor, &self.object.common().actor)?;
    self.object.verify(context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor =
      get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
    let object =
      get_or_fetch_and_insert_post_or_comment(&self.object.object, context, request_counter)
        .await?;
    match object {
      PostOrComment::Post(p) => undo_vote_post(actor, p.deref(), context).await,
      PostOrComment::Comment(c) => undo_vote_comment(actor, c.deref(), context).await,
    }
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
