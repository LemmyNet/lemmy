use crate::{
  activities::{
    following::accept::AcceptFollowCommunity,
    generate_activity_id,
    verify_activity,
    verify_person,
  },
  activity_queue::send_activity_new,
  extensions::context::lemmy_context,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
  ActorType,
};
use activitystreams::activity::kind::FollowType;
use lemmy_api_common::blocking;
use lemmy_apub_lib::{verify_urls_match, ActivityCommonFields, ActivityHandler};
use lemmy_db_queries::Followable;
use lemmy_db_schema::source::{
  community::{Community, CommunityFollower, CommunityFollowerForm},
  person::Person,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowCommunity {
  pub(in crate::activities::following) to: Url,
  pub(in crate::activities::following) object: Url,
  #[serde(rename = "type")]
  pub(in crate::activities::following) kind: FollowType,
  #[serde(flatten)]
  pub(in crate::activities::following) common: ActivityCommonFields,
}

impl FollowCommunity {
  pub async fn send(
    actor: &Person,
    community: &Community,
    context: &LemmyContext,
  ) -> Result<(), LemmyError> {
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: true,
    };
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow(conn, &community_follower_form).ok()
    })
    .await?;

    let id = generate_activity_id(FollowType::Follow)?;
    let follow = FollowCommunity {
      to: community.actor_id(),
      object: community.actor_id(),
      kind: FollowType::Follow,
      common: ActivityCommonFields {
        context: lemmy_context(),
        id: id.clone(),
        actor: actor.actor_id(),
        unparsed: Default::default(),
      },
    };
    let inbox = vec![community.inbox_url.clone().into()];
    send_activity_new(context, &follow, &id, actor, inbox, true).await
  }
}

#[async_trait::async_trait(?Send)]
impl ActivityHandler for FollowCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_urls_match(&self.to, &self.object)?;
    verify_person(&self.common.actor, context, request_counter).await?;
    Ok(())
  }

  async fn receive(
    self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let actor =
      get_or_fetch_and_upsert_person(&self.common.actor, context, request_counter).await?;
    let community =
      get_or_fetch_and_upsert_community(&self.object, context, request_counter).await?;
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: actor.id,
      pending: false,
    };

    // This will fail if they're already a follower, but ignore the error.
    blocking(context.pool(), move |conn| {
      CommunityFollower::follow(conn, &community_follower_form).ok()
    })
    .await?;

    AcceptFollowCommunity::send(self, context).await
  }

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
