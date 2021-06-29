use crate::{activities_new::verify_mod_action, inbox::new_inbox_routing::Activity};
use activitystreams::activity::kind::BlockType;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::{Bannable, Followable};
use lemmy_db_schema::source::community::{
  CommunityFollower,
  CommunityFollowerForm,
  CommunityPersonBan,
  CommunityPersonBanForm,
};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockUserFromCommunity {
  actor: Url,
  to: PublicUrl,
  pub(in crate::activities_new::community) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: BlockType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<BlockUserFromCommunity> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    verify_mod_action(self.inner.actor.clone(), self.inner.cc[0].clone(), context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<BlockUserFromCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.inner.cc[0], context, request_counter).await?;
    let blocked_user =
      get_or_fetch_and_upsert_person(&self.inner.object, context, request_counter).await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: community.id,
      person_id: blocked_user.id,
    };

    blocking(context.pool(), move |conn: &'_ _| {
      CommunityPersonBan::ban(conn, &community_user_ban_form)
    })
    .await??;

    // Also unsubscribe them from the community, if they are subscribed
    let community_follower_form = CommunityFollowerForm {
      community_id: community.id,
      person_id: blocked_user.id,
      pending: false,
    };
    blocking(context.pool(), move |conn: &'_ _| {
      CommunityFollower::unfollow(conn, &community_follower_form)
    })
    .await?
    .ok();

    Ok(())
  }
}
