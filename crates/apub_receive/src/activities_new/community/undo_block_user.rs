use crate::{
  activities_new::{community::block_user::BlockUserFromCommunity, verify_mod_action},
  inbox::new_inbox_routing::Activity,
};
use activitystreams::activity::kind::BlockType;
use lemmy_api_common::blocking;
use lemmy_apub::{
  check_is_apub_id_valid,
  fetcher::{community::get_or_fetch_and_upsert_community, person::get_or_fetch_and_upsert_person},
};
use lemmy_apub_lib::{verify_domains_match, PublicUrl, ReceiveActivity, VerifyActivity};
use lemmy_db_queries::Bannable;
use lemmy_db_schema::source::community::{CommunityPersonBan, CommunityPersonBanForm};
use lemmy_utils::LemmyError;
use lemmy_websocket::LemmyContext;
use url::Url;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoBlockUserFromCommunity {
  actor: Url,
  to: PublicUrl,
  object: Activity<BlockUserFromCommunity>,
  cc: [Url; 1],
  #[serde(rename = "type")]
  kind: BlockType,
}

#[async_trait::async_trait(?Send)]
impl VerifyActivity for Activity<UndoBlockUserFromCommunity> {
  async fn verify(&self, context: &LemmyContext) -> Result<(), LemmyError> {
    verify_domains_match(&self.inner.actor, self.id_unchecked())?;
    check_is_apub_id_valid(&self.inner.actor, false)?;
    verify_mod_action(self.inner.actor.clone(), self.inner.cc[0].clone(), context).await?;
    self.inner.object.verify(context).await
  }
}

#[async_trait::async_trait(?Send)]
impl ReceiveActivity for Activity<UndoBlockUserFromCommunity> {
  async fn receive(
    &self,
    context: &LemmyContext,
    request_counter: &mut i32,
  ) -> Result<(), LemmyError> {
    let community =
      get_or_fetch_and_upsert_community(&self.inner.cc[0], context, request_counter).await?;
    let blocked_user =
      get_or_fetch_and_upsert_person(&self.inner.object.inner.object, context, request_counter)
        .await?;

    let community_user_ban_form = CommunityPersonBanForm {
      community_id: community.id,
      person_id: blocked_user.id,
    };

    blocking(context.pool(), move |conn: &'_ _| {
      CommunityPersonBan::unban(conn, &community_user_ban_form)
    })
    .await??;

    Ok(())
  }
}
