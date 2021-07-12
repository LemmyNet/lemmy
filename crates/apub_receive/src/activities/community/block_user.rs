use crate::activities::{verify_activity, verify_mod_action, verify_person_in_community};
use activitystreams::activity::kind::BlockType;
use lemmy_api_common::blocking;
use lemmy_apub::fetcher::{
  community::get_or_fetch_and_upsert_community,
  person::get_or_fetch_and_upsert_person,
};
use lemmy_apub_lib::{ActivityCommonFields, ActivityHandlerNew, PublicUrl};
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
  to: PublicUrl,
  pub(in crate::activities::community) object: Url,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: BlockType,
  #[serde(flatten)]
  common: ActivityCommonFields,
}

#[async_trait::async_trait(?Send)]
impl ActivityHandlerNew for BlockUserFromCommunity {
  async fn verify(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    verify_activity(self.common())?;
    verify_person_in_community(&self.common.actor, &self.cc, context, request_counter).await?;
    verify_mod_action(&self.common.actor, self.cc[0].clone(), context).await?;
    Ok(())
  }

  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.cc[0], context, request_counter).await?;
    let blocked_user =
      get_or_fetch_and_upsert_person(&self.object, context, request_counter).await?;

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

  fn common(&self) -> &ActivityCommonFields {
    &self.common
  }
}
