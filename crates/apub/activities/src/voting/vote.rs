use crate::{
  check_community_deleted_or_removed,
  generate_activity_id,
  protocol::voting::vote::{Vote, VoteType},
  voting::{vote_comment, vote_post},
};
use activitypub_federation::{
  config::Data,
  fetch::object_id::ObjectId,
  traits::{Activity, Object},
};
use lemmy_api_utils::{
  context::LemmyContext,
  utils::{check_bot_account, check_community_downvote_mode},
};
use lemmy_apub_objects::{
  objects::{PostOrComment, community::ApubCommunity, person::ApubPerson},
  utils::{functions::verify_person_in_community, protocol::InCommunity},
};
use lemmy_utils::error::{LemmyError, LemmyResult};
use url::Url;

impl Vote {
  pub(in crate::voting) fn new(
    object_id: ObjectId<PostOrComment>,
    actor: &ApubPerson,
    community: &ApubCommunity,
    kind: VoteType,
    context: &Data<LemmyContext>,
  ) -> LemmyResult<Vote> {
    Ok(Vote {
      actor: actor.id().clone().into(),
      object: object_id,
      kind: kind.clone(),
      id: generate_activity_id(kind, context)?,
      audience: Some(community.ap_id.clone().into()),
    })
  }
}

#[async_trait::async_trait]
impl Activity for Vote {
  type DataType = LemmyContext;
  type Error = LemmyError;

  fn id(&self) -> &Url {
    &self.id
  }

  fn actor(&self) -> &Url {
    self.actor.inner()
  }

  async fn verify(&self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let actor = self.actor.dereference(context).await?;
    let community = self.community(context).await?;
    check_community_deleted_or_removed(&community)?;
    verify_person_in_community(&self.actor, &community, context).await?;
    check_bot_account(&actor.0)?;
    check_community_downvote_mode(
      Some(self.kind == VoteType::Like),
      &community,
      actor.id,
      &mut context.pool(),
    )
    .await?;
    Ok(())
  }

  async fn receive(self, context: &Data<LemmyContext>) -> LemmyResult<()> {
    let actor = self.actor.dereference(context).await?;
    let object = self.object.dereference(context).await?;

    match object {
      PostOrComment::Left(p) => vote_post(&self.kind, actor, &p, context).await,
      PostOrComment::Right(c) => vote_comment(&self.kind, actor, &c, context).await,
    }
  }
}
